-- Galactic Campaign rewards the game API sends only as an item id. The app's built-in list names the known
-- ones; a row here names a new one, or corrects a listed one, without a release. Added by hand in the
-- Supabase table editor or SQL editor, read by the app.

create table public.campaign_rewards (
  -- The `reward` id on the campaign in the `episodes` snapshot.
  mix_id bigint primary key,
  -- What the in-game Control Center calls its type: Stratagem, Primary weapon, Helmet, Armor, Cape...
  kind text not null,
  -- The item; null while only the type is announced.
  name text,
  -- Which campaign it is for, as a reminder; the app doesn't read it.
  campaign text
);

alter table public.campaign_rewards enable row level security;

revoke all on public.campaign_rewards from anon, authenticated;
grant select on public.campaign_rewards to anon, authenticated;
create policy "public read" on public.campaign_rewards for select to anon, authenticated using (true);
grant select, insert, update, delete on public.campaign_rewards to service_role;
