export type FileInfo = {
  name: string;
  path: string;
  size: number;
  created: string;
  file_type: "converted" | "temp";
};
