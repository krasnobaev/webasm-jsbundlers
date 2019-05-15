import rust from 'rollup-plugin-rust';
import htmlTemplate from 'rollup-plugin-generate-html-template';

export default [
  {
    input: './index.js',
    entry: './index.js',
    output: {
      // dir: 'dist',
      file: 'dist/index.js', // make sure dist folder is exist
      format: 'cjs'
    },
    plugins: [
      htmlTemplate({
        template: './index.html',
        target: 'index.html',
      }),
      rust(),
    ]
  }
];
