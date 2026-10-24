import type { MapDispatches, MapOverlay, MapPlanets, MapQuality } from "../../state";

export const OVERLAY_OPTIONS: { value: MapOverlay; label: string }[] = [
  { value: "regions", label: "Regions" },
  { value: "sectors", label: "Sectors" },
  { value: "galaxy", label: "Galaxy" },
];

export const QUALITY_OPTIONS: { value: MapQuality; label: string }[] = [
  { value: "min", label: "Minimal" },
  { value: "normal", label: "Normal" },
  { value: "max", label: "Maximum" },
];

export const DISPATCH_OPTIONS: { value: MapDispatches; label: string }[] = [
  { value: "3", label: "Last 3" },
  { value: "5", label: "Last 5" },
  { value: "10", label: "Last 10" },
  { value: "72h", label: "72 hours" },
];

export const PLANET_OPTIONS: { value: MapPlanets; label: string }[] = [
  { value: "markers", label: "Markers" },
  { value: "spheres", label: "Spheres" },
];
