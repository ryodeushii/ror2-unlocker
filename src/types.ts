export interface Catalog {
  version: string;
  source: string;
  entries: Entry[];
}

export type EntryKind =
  | "characters"
  | "skills"
  | "achievements"
  | "skins"
  | "artifacts"
  | "items"
  | "equipment";

export interface Entry {
  id: string;
  kind: EntryKind;
  label: string;
  description: string;
  achievement: string | null;
  unlock: string | null;
  discovery: string | null;
  expansion: string;
}

export interface ProfileSnapshot {
  path: string;
  name: string;
  coins: number;
  revision: string;
  unlocked_ids: string[];
}

export interface ChangeRequest {
  path: string;
  revision: string;
  selection: string[];
  coins: number | null;
}

export interface FieldChange {
  field: string;
  added: string[];
}

export interface Preview {
  changes: FieldChange[];
  coins_before: number;
  coins_after: number;
  total_additions: number;
}

export interface SaveResult {
  profile: ProfileSnapshot;
  backup_path: string | null;
  preview: Preview;
}
