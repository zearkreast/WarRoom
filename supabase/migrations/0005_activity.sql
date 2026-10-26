-- Helldiver activity beyond the fronts: each SEST day's busiest and quietest moment galaxy-wide, Helldivers
-- on planets with no campaign, and each city's readings. Written by the collector, read by the app.

-- One row per SEST (UTC) day: the most and fewest Helldivers online, and when. Kept for good.
create table public.war_days (
  day date primary key,
  peak integer not null,
  peak_at timestamptz not null,
  low integer not null,
  low_at timestamptz not null
);

-- Days before this migration come from war_samples, which keeps one row per 15 minutes once it's older than
-- 48 hours, so a short-lived peak or low on those days may be missing.
insert into public.war_days (day, peak, peak_at, low, low_at)
select
  (s.at at time zone 'UTC')::date,
  max(s.players),
  (array_agg(s.at order by s.players desc, s.at))[1],
  min(s.players),
  (array_agg(s.at order by s.players, s.at))[1]
from public.war_samples s
where s.players > 0
group by 1;

-- Folds one war reading into its day. Called by the collector's war task on every poll.
create function public.note_war_day(reading_at timestamptz, online integer) returns void
language sql
security invoker
set search_path = ''
as $$
  insert into public.war_days as d (day, peak, peak_at, low, low_at)
  values ((reading_at at time zone 'UTC')::date, online, reading_at, online, reading_at)
  on conflict (day) do update set
    peak = greatest(d.peak, excluded.peak),
    peak_at = case when excluded.peak > d.peak then excluded.peak_at else d.peak_at end,
    low = least(d.low, excluded.low),
    low_at = case when excluded.low < d.low then excluded.low_at else d.low_at end;
$$;

-- Helldivers on a planet with no campaign (samples only covers contested planets), e.g. Super Earth on a
-- holiday. Readings with nobody there aren't stored.
create table public.planet_players (
  planet smallint not null,
  at timestamptz not null,
  players integer not null,
  primary key (planet, at)
);
create index planet_players_at on public.planet_players (at);

-- A city's readings: its Helldivers, and its liberation while it's fought over (null otherwise).
create table public.region_samples (
  planet smallint not null,
  region smallint not null,
  at timestamptz not null,
  players integer not null,
  percent real,
  primary key (planet, region, at)
);
create index region_samples_at on public.region_samples (at);

alter table public.war_days enable row level security;
alter table public.planet_players enable row level security;
alter table public.region_samples enable row level security;

revoke all on public.war_days, public.planet_players, public.region_samples from anon, authenticated;
grant select on public.war_days, public.planet_players, public.region_samples to anon, authenticated;
create policy "public read" on public.war_days for select to anon, authenticated using (true);
create policy "public read" on public.planet_players for select to anon, authenticated using (true);
create policy "public read" on public.region_samples for select to anon, authenticated using (true);
grant select, insert, update, delete on public.war_days, public.planet_players, public.region_samples to service_role;

revoke execute on function public.note_war_day(timestamptz, integer) from public, anon, authenticated;
grant execute on function public.note_war_day(timestamptz, integer) to service_role;

-- Keeps the first reading per planet (and city) per 15-minute bucket once it's older than 48 hours, like
-- compact_samples().
create function public.compact_activity() returns integer
language sql
security invoker
set search_path = ''
as $$
  with idle as (
    delete from public.planet_players s
    where s.at < now() - interval '48 hours'
      and exists (
        select 1 from public.planet_players k
        where k.planet = s.planet
          and k.at < s.at
          and date_bin('15 minutes', k.at, timestamptz '2000-01-01') = date_bin('15 minutes', s.at, timestamptz '2000-01-01')
      )
    returning 1
  ),
  cities as (
    delete from public.region_samples s
    where s.at < now() - interval '48 hours'
      and exists (
        select 1 from public.region_samples k
        where k.planet = s.planet
          and k.region = s.region
          and k.at < s.at
          and date_bin('15 minutes', k.at, timestamptz '2000-01-01') = date_bin('15 minutes', s.at, timestamptz '2000-01-01')
      )
    returning 1
  )
  select ((select count(*) from idle) + (select count(*) from cities))::integer;
$$;

revoke execute on function public.compact_activity() from public, anon, authenticated;
grant execute on function public.compact_activity() to service_role;
