// src/lib/stores/formats.ts

import { writable } from "svelte/store";
import type { Format } from "$lib/types/format";

export const formats = writable<Format[]>([]);
export const selectedFormat = writable<Format | null>(null);
export const formatsLoading = writable(false);
export const formatsError = writable<string | null>(null);
export const formatsLoaded = writable(false);
