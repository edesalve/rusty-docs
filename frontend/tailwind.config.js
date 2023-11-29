/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './src/pages/**/*.{js,ts,jsx,tsx,mdx}',
    './src/components/**/*.{js,ts,jsx,tsx,mdx}',
    './src/app/**/*.{js,ts,jsx,tsx,mdx}',
  ],
  theme: {
    extend: {
      colors: {
        docBlue: '#2dbfb8',
        docGreen: '#2cab63',
        docGrey1: '#717171',
        docGrey2: '#505050',
        docGrey3: '#363636',
        docGrey4: '#2a2a2a',
        docViolet: '#b78cf2',
        docWhite: '#dddddd',
        docYellow: '#d2991d',
      },
    },
  },
  plugins: [],
}
