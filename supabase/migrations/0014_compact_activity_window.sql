-- Ranks each 15-minute bucket in one pass over the primary key. A self-join here gets hashed by planet alone,
-- which took over 10 s on two weeks of readings, past the collector's 8-second limit.
-- The scan starts at the bucket holding the 14-day line, so that bucket is ranked whole.

create or replace function public.compact_activity() returns integer
language sql
security invoker
set search_path = ''
as $$
  with idle_ranked as (
    select planet, at,
      row_number() over (
        partition by planet, date_bin('15 minutes', at, timestamptz '2000-01-01') order by at
      ) as n
    from public.planet_players
    where at < now() - interval '48 hours'
      and at >= date_bin('15 minutes', now() - interval '14 days', timestamptz '2000-01-01')
  ),
  idle as (
    delete from public.planet_players p
    using idle_ranked r
    where r.n > 1
      and r.at >= now() - interval '14 days'
      and p.planet = r.planet
      and p.at = r.at
    returning 1
  ),
  cities_ranked as (
    select planet, region, at,
      row_number() over (
        partition by planet, region, date_bin('15 minutes', at, timestamptz '2000-01-01') order by at
      ) as n
    from public.region_samples
    where at < now() - interval '48 hours'
      and at >= date_bin('15 minutes', now() - interval '14 days', timestamptz '2000-01-01')
  ),
  cities as (
    delete from public.region_samples s
    using cities_ranked r
    where r.n > 1
      and r.at >= now() - interval '14 days'
      and s.planet = r.planet
      and s.region = r.region
      and s.at = r.at
    returning 1
  )
  select ((select count(*) from idle) + (select count(*) from cities))::integer;
$$;
