/** @type {import('tailwindcss').Config} */
module.exports = {
    content: {
        files: ["*.html", "./src/**/*.rs"],
    },
    safelist: [
        'text-blue-400',
        'text-green-400',
        'text-orange-400',
        'bg-indigo-500',
        'bg-zinc-700'
    ],
    theme: {
        extend: {},
    },
    plugins: [],
}