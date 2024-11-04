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
        'bg-zinc-700',
        'text-rose-500',
        'text-blue-500',
        'bg-rose-500',
        'bg-rose-600',
        'bg-blue-600',
        'border-rose-600',
        'border-blue-600',
    ],
    theme: {
        extend: {},
    },
    plugins: [],
}