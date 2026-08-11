/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      fontFamily: {
        sans: ['Inter', 'system-ui', 'sans-serif'],
        display: ['Plus Jakarta Sans', 'Inter', 'sans-serif'],
        mono: ['JetBrains Mono', 'monospace'],
      },
      colors: {
        vel: {
          dark: '#060913',
          card: 'rgba(15, 23, 42, 0.7)',
          orange: '#f97316',
          amber: '#f59e0b',
        }
      }
    },
  },
  plugins: [],
}
