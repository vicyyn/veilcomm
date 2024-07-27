module.exports = {
  content: [
    "./src/**/*.{js,jsx,ts,tsx}",
    "./public/index.html",
  ],
  theme: {
    extend: {
      backgroundImage: {
        'grid-pattern': `linear-gradient(#e8e8e8 1px, transparent 1px),
                         linear-gradient(90deg, #e8e8e8 1px, transparent 1px)`,
      },
      backgroundSize: {
        'grid-size': '20px 20px',
      },
    },
  },
  plugins: [],
}