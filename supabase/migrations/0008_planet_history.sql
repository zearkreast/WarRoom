-- A planet's readings over up to 30 days for the planet page's longer chart ranges: the last reading in each
-- bucket, so a month reads a few hundred rows instead of every one kept.

create function public.planet_history(target smallint, hours integer default 168)
returns table (planet smallint, at timestamptz, owner smallint, percent real, players integer, defending boolean)
language sql
stable
security invoker
set search_path = ''
as $$
  with step as (
    select case
      when hours <= 48 then interval '15 minutes'
      when hours <= 168 then interval '1 hour'
      else interval '3 hours'
    end as width
  )
  select distinct on (date_bin(step.width, s.at, timestamptz '2000-01-01'))
    s.planet, s.at, s.owner, s.percent, s.players, s.defending
  from public.samples s, step
  where s.planet = target
    and s.at >= now() - make_interval(hours => least(greatest(hours, 1), 720))
  order by date_bin(step.width, s.at, timestamptz '2000-01-01'), s.at desc;
$$;

revoke execute on function public.planet_history(smallint, integer) from public;
grant execute on function public.planet_history(smallint, integer) to anon, authenticated, service_role;
