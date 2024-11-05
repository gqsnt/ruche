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
        'text-red-500',
        'text-blue-500',
        'bg-red-500',
        'bg-blue-400',
        'bg-red-400',
        'text-blue-300',
        'bg-red-900',
        'bg-blue-900',
        'bg-red-600',
        'bg-blue-600',
        'border-red-500',
        'border-blue-500',
        'text-red-300'
    ],
    "colors": {
        "red": {
            50: "#FDEDEF",
            100: "#FBDBDF",
            200: "#F6B2BB",
            300: "#F18D9B",
            400: "#EC6476",
            500: "#E84057",
            600: "#D11932",
            700: "#9F1326",
            800: "#690D19",
            900: "#37070D",
            950: "#1B0306"
        },
        "blue": {
            50: "#EDF2FD",
            100: "#DBE5FA",
            200: "#BCCFF6",
            300: "#98B5F1",
            400: "#749CEC",
            500: "#5384E8",
            600: "#1D5DDC",
            700: "#1646A6",
            800: "#0F3070",
            900: "#071736",
            950: "#040B1B"
        }
    },
    theme: {
        extend: {},
    },
    plugins: [],
}