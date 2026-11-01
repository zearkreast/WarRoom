-- Each planet's rows are spread across most of `samples`, so without this a planet's times under attack take
-- thousands of page reads and can pass the 3-second limit on anonymous reads.
create index samples_defending on public.samples (planet, at) where defending is true;
