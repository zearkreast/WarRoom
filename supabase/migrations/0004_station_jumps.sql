-- Every FTL jump a space station makes: where it left and where it arrived.
-- Written by the collector's stations task when a station's planet changes, read by the DSS page.

create table public.station_jumps (
  station bigint not null,
  at timestamptz not null,
  from_planet integer,
  to_planet integer not null,
  primary key (station, at)
);

alter table public.station_jumps enable row level security;

revoke all on public.station_jumps from anon, authenticated;
grant select on public.station_jumps to anon, authenticated;
create policy "public read" on public.station_jumps for select to anon, authenticated using (true);
grant select, insert, update, delete on public.station_jumps to service_role;
