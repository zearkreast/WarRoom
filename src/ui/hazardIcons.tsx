import {
  Activity,
  CloudFog,
  CloudLightning,
  CloudRain,
  Droplets,
  Flame,
  Mountain,
  Snowflake,
  Sparkles,
  Sun,
  TriangleAlert,
  Wind,
  type LucideIcon,
} from "lucide-react";

const ICONS: [RegExp, LucideIcon][] = [
  [/heat|fire|scorch|volcan/i, Flame],
  [/cold|blizzard|snow|ice|frost/i, Snowflake],
  [/rain/i, CloudRain],
  [/fog/i, CloudFog],
  [/sand|dust|wind/i, Wind],
  [/ion|electric|lightning|storm/i, CloudLightning],
  [/meteor|ember/i, Sparkles],
  [/acid|toxic|spore/i, Droplets],
  [/tremor|quake/i, Activity],
  [/sun|solar/i, Sun],
  [/rock|cliff|terrain/i, Mountain],
];

export function hazardIcon(name: string): LucideIcon {
  return ICONS.find(([re]) => re.test(name))?.[1] ?? TriangleAlert;
}
