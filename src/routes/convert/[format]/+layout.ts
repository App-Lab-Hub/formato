// src/routes/convert/[format]/+layout.ts

import type { LayoutLoad } from "./$types";
import { getAvailability } from "$lib/data/availability";

export const load: LayoutLoad = async ({ params, parent }) => {
  // Ждём данные от родительского layout
  const parentData = await parent();

  const formatId = params.format;
  // console.log("FOMAT ID=>", formatId);
  // Находим формат в загруженных данных
  const format = parentData.formats?.find((f: any) => f.id === formatId);

  // Получаем доступность конвертации
  let availability = null;
  if (format) {
    try {
      availability = await getAvailability(format.formatType);
      // console.log(availability);
    } catch (e) {
      // console.error("Failed to get availability:", e);
    }
  }

  return {
    format,
    availability,
  };
};
