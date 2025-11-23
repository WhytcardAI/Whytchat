/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // WhytChat Light Theme (Based on User Image)
        background: '#f3f4f6', // Light Gray background
        surface: '#ffffff',    // White surface
        primary: '#ea580c',    // Orange 600 (Action color)
        secondary: '#8b5cf6',  // Violet 500 (Agent/Persona)
        text: '#1f2937',       // Gray 800
        muted: '#9ca3af',      // Gray 400
        border: '#e5e7eb',     // Gray 200
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'monospace'],
      },
    },
  },
  plugins: [],
}
