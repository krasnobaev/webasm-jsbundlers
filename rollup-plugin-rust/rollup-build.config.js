import livereload from 'rollup-plugin-livereload';
import rust from 'rollup-plugin-rust';
import serve from 'rollup-plugin-serve';
import htmlTemplate from 'rollup-plugin-generate-html-template';

export default [
  {
    input: './index.js',
    entry: './index.js',
    output: {
      // dir: 'dist',
      file: 'dist/index.js',
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
