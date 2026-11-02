-- Every Major Order seen, with its text, tasks and latest progress, and its progress at each poll, so the app
-- can chart a whole order and keep past ones. Written by the collector's assignments task.

create table public.orders (
  id bigint primary key,
  title text,
  briefing text,
  expires timestamptz not null,
  -- The order's tasks and progress as the API gives them.
  tasks jsonb not null,
  progress jsonb not null,
  first_seen timestamptz not null default now(),
  last_seen timestamptz not null
);

create table public.order_progress (
  id bigint not null,
  at timestamptz not null,
  progress jsonb not null,
  primary key (id, at)
);

alter table public.orders enable row level security;
alter table public.order_progress enable row level security;

revoke all on public.orders from anon, authenticated;
revoke all on public.order_progress from anon, authenticated;
grant select on public.orders to anon, authenticated;
grant select on public.order_progress to anon, authenticated;
create policy "public read" on public.orders for select to anon, authenticated using (true);
create policy "public read" on public.order_progress for select to anon, authenticated using (true);
grant select, insert, update, delete on public.orders to service_role;
grant select, insert, update, delete on public.order_progress to service_role;
