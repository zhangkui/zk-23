import { createDOMRenderer, render } from '@builder.io/qwik';
import Root from './root';

export default () => {
  const renderer = createDOMRenderer(document);
  return render(<Root />, {
    renderer,
  });
};
