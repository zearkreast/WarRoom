-- Galaxy-wide trend per war poll: impact multiplier and Helldivers per front.
-- Written by the collector's war task, read by the Overview and Planets charts.

create table public.war_samples (
  at timestamptz primary key,
  impact real not null,
  players integer not null,
  terminids integer not null,
  automatons integer not null,
  illuminate integer not null
);

alter table public.war_samples enable row level security;

revoke all on public.war_samples from anon, authenticated;
grant select on public.war_samples to anon, authenticated;
create policy "public read" on public.war_samples for select to anon, authenticated using (true);
grant select, insert, update, delete on public.war_samples to service_role;

-- Keep one row per 15 minutes once it's older than 48 hours, like compact_samples().
create function public.compact_war_samples() returns integer
language sql
security invoker
set search_path = ''
as $$
  with doomed as (
    delete from public.war_samples s
    where s.at < now() - interval '48 hours'
      and exists (
        select 1 from public.war_samples k
        where k.at < s.at
          and date_bin('15 minutes', k.at, timestamptz '2000-01-01') = date_bin('15 minutes', s.at, timestamptz '2000-01-01')
      )
    returning 1
  )
  select count(*)::integer from doomed;
$$;

revoke execute on function public.compact_war_samples() from public, anon, authenticated;
grant execute on function public.compact_war_samples() to service_role;
