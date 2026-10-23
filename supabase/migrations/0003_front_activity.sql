-- Helldivers on each front over the last hours, in 15-minute buckets, from the samples the collector
-- already keeps. Lets the Overview's activity chart show history at once instead of filling over a day.
--
-- Liberations are summed per enemy (the planet's owner). Defenses come back per planet, because the
-- samples don't record the attacker; the app attributes them from the live planet data.

create function public.front_activity(hours integer default 24)
returns table (bucket timestamptz, owner smallint, planet smallint, players integer)
language sql
stable
security invoker
set search_path = ''
as $$
  with readings as (
    select
      date_bin('15 minutes', s.at, timestamptz '2000-01-01') as bucket,
      s.planet,
      s.defending,
      (array_agg(s.owner order by s.at desc))[1] as owner,
      round(avg(s.players))::integer as players
    from public.samples s
    where s.at >= now() - make_interval(hours => least(greatest(hours, 1), 48))
    group by 1, 2, 3
  )
  select r.bucket, r.owner, case when r.defending then r.planet end, sum(r.players)::integer
  from readings r
  group by r.bucket, r.owner, case when r.defending then r.planet end
  order by r.bucket;
$$;

revoke execute on function public.front_activity(integer) from public;
grant execute on function public.front_activity(integer) to anon, authenticated, service_role;
