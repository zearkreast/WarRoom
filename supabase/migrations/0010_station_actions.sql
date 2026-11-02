-- Every change in a space station's tactical actions: when each went active, cooled down or began gathering
-- contributions again. Written by the collector's stations task, read by the Space stations page.

create table public.station_actions (
  station bigint not null,
  action bigint not null,
  at timestamptz not null,
  -- The API's status: 1 preparing, 2 active, 3 cooling down.
  status smallint not null,
  expires timestamptz,
  primary key (station, action, at)
);

alter table public.station_actions enable row level security;

revoke all on public.station_actions from anon, authenticated;
grant select on public.station_actions to anon, authenticated;
create policy "public read" on public.station_actions for select to anon, authenticated using (true);
grant select, insert, update, delete on public.station_actions to service_role;
