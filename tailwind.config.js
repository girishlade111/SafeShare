/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        bg: {
          DEFAULT: "#0b0d10",
          panel: "#111418",
          soft: "#161a20",
        },
        ink: {
          DEFAULT: "#e6e8eb",
          muted: "#9aa3ad",
          dim: "#6b7280",
        },
        accent: {
          DEFAULT: "#7c9cff",
          soft: "#1a2238",
        },
        line: "#222831",
      },
      fontFamily: {
        sans: [
          "ui-sans-serif",
          "system-ui",
          "-apple-system",
          "Segoe UI",
          "Inter",
          "sans-serif",
        ],
      },
    },
  },
  plugins: [],
};