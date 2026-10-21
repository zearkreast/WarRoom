-- WarRoom: Galactic War data written by the collector, read by the app.
-- Factions are smallint codes: 1 Humans, 2 Terminids, 3 Automaton, 4 Illuminate.

-- Latest copy of each API resource, overwritten on every poll (constant size).
create table public.snapshots (
  kind text primary key,
  data jsonb not null,
  fetched_at timestamptz not null default now()
);

-- Readings of contested planets. Thinned to one per 15 min after 48 h by compact_samples().
create table public.samples (
  planet smallint not null,
  at timestamptz not null,
  owner smallint not null,
  percent real not null,
  players integer not null,
  defending boolean not null,
  primary key (planet, at)
);
create index samples_at on public.samples (at);

-- Who took which planet, when.
create table public.owner_changes (
  id bigint generated always as identity primary key,
  planet smallint not null,
  planet_name text not null,
  from_owner smallint not null,
  to_owner smallint not null,
  at timestamptz not null
);
create index owner_changes_planet_at on public.owner_changes (planet, at desc);

-- Collector state: last owner seen per planet. Not exposed to the app.
create table public.planet_owners (
  planet smallint primary key,
  owner smallint not null
);

alter table public.snapshots enable row level security;
alter table public.samples enable row level security;
alter table public.owner_changes enable row level security;
alter table public.planet_owners enable row level security;

-- The app (publishable key = anon role) may only read. Revoke first in case default
-- privileges granted more.
revoke all on public.snapshots, public.samples, public.owner_changes, public.planet_owners from anon, authenticated;
grant select on public.snapshots, public.samples, public.owner_changes to anon, authenticated;
create policy "public read" on public.snapshots for select to anon, authenticated using (true);
create policy "public read" on public.samples for select to anon, authenticated using (true);
create policy "public read" on public.owner_changes for select to anon, authenticated using (true);

-- The collector (secret key = service_role) writes everything.
grant select, insert, update, delete on public.snapshots, public.samples, public.owner_changes, public.planet_owners to service_role;

-- Keep the first sample per planet per 15-minute bucket once it's older than 48 hours.
create function public.compact_samples() returns integer
language sql
security invoker
set search_path = ''
as $$
  with doomed as (
    delete from public.samples s
    where s.at < now() - interval '48 hours'
      and exists (
        select 1 from public.samples k
        where k.planet = s.planet
          and k.at < s.at
          and date_bin('15 minutes', k.at, timestamptz '2000-01-01') = date_bin('15 minutes', s.at, timestamptz '2000-01-01')
      )
    returning 1
  )
  select count(*)::integer from doomed;
$$;

revoke execute on function public.compact_samples() from public, anon, authenticated;
grant execute on function public.compact_samples() to service_role;
