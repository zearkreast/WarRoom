-- Pictures of Galactic Campaigns and their Major Orders, one WebP per game image id (`<id>.webp`). The game's
-- API names a picture only by id, so they are uploaded by hand; the app shows those it finds.

insert into storage.buckets (id, name, public, file_size_limit, allowed_mime_types)
values ('campaign-art', 'campaign-art', true, 1048576, array['image/webp'])
on conflict (id) do nothing;
