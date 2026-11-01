-- Thinning reads only the last two weeks, and only inside a reading's own 15-minute bucket. Hashing every stored
-- reading each hour outgrew the collector's 8-second limit once years of history were imported.

create or replace function public.compact_samples() returns integer
language sql
security invoker
set search_path = ''
as $$
  with doomed as (
    delete from public.samples s
    where s.at < now() - interval '48 hours'
      and s.at >= now() - interval '14 days'
      and exists (
        select 1 from public.samples k
        where k.planet = s.planet
          and k.at < s.at
          and k.at >= date_bin('15 minutes', s.at, timestamptz '2000-01-01')
      )
    returning 1
  )
  select count(*)::integer from doomed;
$$;

create or replace function public.compact_war_samples() returns integer
language sql
security invoker
set search_path = ''
as $$
  with doomed as (
    delete from public.war_samples s
    where s.at < now() - interval '48 hours'
      and s.at >= now() - interval '14 days'
      and exists (
        select 1 from public.war_samples k
        where k.at < s.at
          and k.at >= date_bin('15 minutes', s.at, timestamptz '2000-01-01')
      )
    returning 1
  )
  select count(*)::integer from doomed;
$$;

create or replace function public.compact_activity() returns integer
language sql
security invoker
set search_path = ''
as $$
  with idle as (
    delete from public.planet_players s
    where s.at < now() - interval '48 hours'
      and s.at >= now() - interval '14 days'
      and exists (
        select 1 from public.planet_players k
        where k.planet = s.planet
          and k.at < s.at
          and k.at >= date_bin('15 minutes', s.at, timestamptz '2000-01-01')
      )
    returning 1
  ),
  cities as (
    delete from public.region_samples s
    where s.at < now() - interval '48 hours'
      and s.at >= now() - interval '14 days'
      and exists (
        select 1 from public.region_samples k
        where k.planet = s.planet
          and k.region = s.region
          and k.at < s.at
          and k.at >= date_bin('15 minutes', s.at, timestamptz '2000-01-01')
      )
    returning 1
  )
  select ((select count(*) from idle) + (select count(*) from cities))::integer;
$$;
