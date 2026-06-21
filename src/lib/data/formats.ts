import {
  FileBraces,
  FileText,
  FileSpreadsheet,
  FileCode,
  FileJson,
  Table,
  AlignLeft,
  Grid3x3,
  ListOrdered,
  Braces,
  Globe,
} from "lucide-svelte";

export const formats = [
  {
    id: "json",
    name: "JSON",
    extensions: ["json", "hjson"],
    description:
      "JavaScript Object Notation — легковесный формат обмена данными, основанный на синтаксисе JavaScript",
    icon: FileBraces,
    color: "from-yellow-500/30 to-amber-500/15",
    glow: "shadow-yellow-500/20",
    textColor: "text-yellow-400",
    borderHover: "hover:border-yellow-500/60",
  },
  {
    id: "yaml",
    name: "YAML",
    extensions: ["yaml", "yml"],
    description:
      "YAML Ain't Markup Language — человекочитаемый формат сериализации данных, популярный в конфигурациях и DevOps",
    icon: FileText,
    color: "from-blue-500/30 to-cyan-500/15",
    glow: "shadow-blue-500/20",
    textColor: "text-blue-400",
    borderHover: "hover:border-blue-500/60",
  },
  {
    id: "csv",
    name: "CSV",
    extensions: ["csv", "tsv"],
    description:
      "Comma-Separated Values — табличный формат для хранения и обмена данными между базами, Excel и аналитическими системами",
    icon: FileSpreadsheet,
    color: "from-green-500/30 to-emerald-500/15",
    glow: "shadow-green-500/20",
    textColor: "text-green-400",
    borderHover: "hover:border-green-500/60",
  },
  {
    id: "xml",
    name: "XML",
    extensions: ["xml"],
    description:
      "Extensible Markup Language — универсальный язык разметки с древовидной структурой, широко используется в API, SOAP и конфигурациях",
    icon: FileCode,
    color: "from-orange-500/30 to-red-500/15",
    glow: "shadow-orange-500/20",
    textColor: "text-orange-400",
    borderHover: "hover:border-orange-500/60",
  },
  {
    id: "toml",
    name: "TOML",
    extensions: ["toml"],
    description:
      "Tom's Obvious Minimal Language — минималистичный формат конфигураций с чёткой структурой, любимец Rust-сообщества",
    icon: AlignLeft,
    color: "from-orange-400/30 to-yellow-500/15",
    glow: "shadow-orange-400/20",
    textColor: "text-orange-400",
    borderHover: "hover:border-orange-400/60",
  },
  {
    id: "ini",
    name: "INI",
    extensions: ["ini", "cfg", "conf"],
    description:
      "Простейший формат конфигурационных файлов с секциями и парами ключ-значение, используется повсеместно",
    icon: ListOrdered,
    color: "from-gray-400/30 to-slate-500/15",
    glow: "shadow-gray-400/20",
    textColor: "text-gray-400",
    borderHover: "hover:border-gray-400/60",
  },
  {
    id: "markdown",
    name: "Markdown",
    extensions: ["md", "markdown", "mdown", "mkd"],
    description:
      "Легковесный язык разметки для форматирования текста, конвертируется в HTML, PDF и другие форматы",
    icon: Braces,
    color: "from-purple-500/30 to-violet-500/15",
    glow: "shadow-purple-500/20",
    textColor: "text-purple-400",
    borderHover: "hover:border-purple-500/60",
  },
  {
    id: "html",
    name: "HTML",
    extensions: ["html", "htm"],
    description:
      "HyperText Markup Language — стандартный язык веб-разметки, основа всех веб-страниц и шаблонов",
    icon: Globe,
    color: "from-orange-500/30 to-red-500/15",
    glow: "shadow-orange-500/20",
    textColor: "text-orange-300",
    borderHover: "hover:border-orange-500/60",
  },
];

export type Format = (typeof formats)[number];
