-- Mission, kill and death counters once an hour, for the galaxy (planet -1) and each planet with a campaign, so
-- the app can tell the last day from the all-time totals the game gives. Written by the collector's planets and
-- war tasks; trim_combat_stats(), run by its hourly compaction, keeps 8 days.

create table public.combat_stats (
  planet smallint not null,
  at timestamptz not null,
  missions_won bigint not null,
  missions_lost bigint not null,
  terminid_kills bigint not null,
  automaton_kills bigint not null,
  illuminate_kills bigint not null,
  deaths bigint not null,
  primary key (planet, at)
);

alter table public.combat_stats enable row level security;

revoke all on public.combat_stats from anon, authenticated;
grant select on public.combat_stats to anon, authenticated;
create policy "public read" on public.combat_stats for select to anon, authenticated using (true);
grant select, insert, update, delete on public.combat_stats to service_role;

create function public.trim_combat_stats() returns integer
language sql
security invoker
set search_path = ''
as $$
  with gone as (
    delete from public.combat_stats where at < now() - interval '8 days' returning 1
  )
  select count(*)::integer from gone;
$$;

revoke execute on function public.trim_combat_stats() from public, anon, authenticated;
grant execute on function public.trim_combat_stats() to service_role;
