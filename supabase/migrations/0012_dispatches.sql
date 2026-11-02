-- Every dispatch the API has offered, so Strohmann News can reach past the newest 50 kept in the snapshot.
-- Written by the collector's dispatches task whenever the list changes; a corrected text replaces the old one.

create table public.dispatches (
  id bigint primary key,
  published timestamptz not null,
  kind integer not null,
  message text not null
);

alter table public.dispatches enable row level security;

revoke all on public.dispatches from anon, authenticated;
grant select on public.dispatches to anon, authenticated;
create policy "public read" on public.dispatches for select to anon, authenticated using (true);
grant select, insert, update, delete on public.dispatches to service_role;
