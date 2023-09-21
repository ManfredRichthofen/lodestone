// eslint-disable-next-line @typescript-eslint/no-var-requires
const defaultTheme = require('tailwindcss/defaultTheme');
/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ['./pages/**/*.{js,ts,jsx,tsx}', './src/**/*.{js,ts,jsx,tsx}'],
  theme: {
    colors: {
      transparent: 'transparent', //! what
      current: 'currentColor',
      blue: {
        150: '#69C9FF',
        200: '#59B2F3',
        300: '#1D8EB2',
        DEFAULT: '#1D8EB2',
        400: '#037AA0',
        500: '#13668A',
        600: '#334675',
        faded: '#3C85BA',
      },
      green: {
        200: '#2AF588',
        300: '#6DD277',
        DEFAULT: '#6DD277',
        400: '#3DAE5E',
        faded: '#61AE32',
        enabled: '#48F077',
      },
      red: {
        200: '#FF5C5C',
        300: '#CF4545',
        DEFAULT: '#CF4545',
        400: '#B63737',
        faded: '#AE3232',
      },
      yellow: {
        300: '#EFB440',
        DEFAULT: '#EFB440',
        faded: '#AE8B32',
      },
      gray: {
        300: '#E3E3E4',
        400: '#A5A5AC',
        500: '#767A82',
        550: '#67686A',
        600: '#44464B',
        700: '#36393F',
        750: '#303338',
        800: '#2B2D32',
        850: '#26282C',
        875: '#212327',
        900: '#1D1E21',
        faded: '#A5A5AC',
      },
      fade: {
        700: '#d1d1da19',
      },
      white: '#FFFFFF',
      violet: '#8736C7',
      ultramarine: '#273EB9',
    },
    fontFamily: {
      sans: ['Satoshi', ...defaultTheme.fontFamily.sans],
      heading: ['Chillax', ...defaultTheme.fontFamily.sans],
      title: ['Clash Grotesk', ...defaultTheme.fontFamily.sans],
      minecraft: ['Minecraft Changed', ...defaultTheme.fontFamily.sans],
      mono: ['JetBrains Mono', ...defaultTheme.fontFamily.mono],
    },
    fontSize: {
      //! note: don't use fractional-px fonts
      caption: '0.625rem',
      small: '0.75rem',
      medium: '0.875rem',
      h3: '1rem',
      h2: '1.25rem',
      h1: '1.75rem',
      title: '2.625rem',
      '2xlarge': '2rem',
      '3xlarge': '2.5rem',
      '4xlarge': '3rem',
      '5xlarge': '3.5rem',
      '6xlarge': '4rem',
    },
    letterSpacing: {
      tight: '-0.04em',
      medium: '-0.01em',
      normal: '0',
      wide: '0.04em',
    },
    dropShadow: {
      sm: '0 0 transparent',
      md: '0 3px 6px #111114',
      lg: '0 8px 24px #111114',
      xl: '0 12px 48px #111114',
    },
    fontWeight: {
      medium: 450,
      mediumbold: 500,
      bold: 550,
      extrabold: 700,
    },
    extend: {
      transitionProperty: {
        //! animation stuff, added some extra transition animations (do we even ues these?)
        height: 'height',
        width: 'width',
        spacing: 'margin, padding',
        dimensions: 'height, width',
      },
    },
  },
  plugins: [
    function ({ addVariant }) {
      addVariant('child', '& > *');
    },
    require('@tailwindcss/container-queries'),
    require('@headlessui/tailwindcss')({ prefix: 'ui' }),
  ],
};
