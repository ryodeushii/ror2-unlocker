import type {
  Catalog,
  ChangeRequest,
  Preview,
  ProfileSnapshot,
  SaveResult,
} from "./types";

export interface DownloadResult {
  kind: "downloads";
  profile: ProfileSnapshot;
  preview: Preview;
  xml: string;
  original: Uint8Array<ArrayBuffer>;
  filename: string;
}

export interface NativeSaveResult extends SaveResult {
  kind: "saved";
}

export interface ProfileBackend {
  getCatalog(): Promise<Catalog>;
  pickProfile(input: HTMLInputElement): Promise<ProfileSnapshot | null>;
  preview(args: ChangeRequest): Promise<Preview>;
  save(args: ChangeRequest): Promise<DownloadResult | NativeSaveResult>;
}
