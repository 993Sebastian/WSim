import { landName } from "../format";

/** Country keys of the data files, sorted by German name. */
export const LAENDER: string[] = Object.keys(import.meta.glob("../../../data/laender/*.yaml"))
  .map((p) => p.split("/").pop()!.replace(".yaml", ""))
  .sort((a, b) => landName(a).localeCompare(landName(b), "de"));
