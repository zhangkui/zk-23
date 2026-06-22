import { component$, Slot, $, useOnDocument } from '@builder.io/qwik';

interface ModalProps {
  isOpen: boolean;
  title: string;
  onClose$: () => void;
  size?: 'sm' | 'md' | 'lg' | 'xl';
  showFooter?: boolean;
  footerActions?: {
    label: string;
    onClick$: () => void;
    variant?: 'primary' | 'secondary' | 'danger';
    disabled?: boolean;
  }[];
}

export default component$<ModalProps>(({
  isOpen,
  title,
  onClose$,
  size = 'md',
  showFooter = false,
  footerActions = [],
}) => {
  const handleKeyDown = $((e: KeyboardEvent) => {
    if (e.key === 'Escape' && isOpen) {
      onClose$();
    }
  });

  useOnDocument('keydown', handleKeyDown);

  if (!isOpen) return null;

  const sizeClasses = {
    sm: 'max-w-sm',
    md: 'max-w-md',
    lg: 'max-w-2xl',
    xl: 'max-w-4xl',
  };

  const variantClasses = {
    primary: 'btn-primary',
    secondary: 'btn-secondary',
    danger: 'btn-danger',
  };

  return (
    <div class="modal-overlay" onClick$={onClose$}>
      <div
        class={`modal ${sizeClasses[size]}`}
        onClick$={(e) => e.stopPropagation()}
      >
        <div class="modal-header">
          <h3 class="text-lg font-semibold text-gray-900">{title}</h3>
          <button
            class="text-gray-400 hover:text-gray-600 transition-colors p-1"
            onClick$={onClose$}
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
        <div class="modal-body">
          <Slot />
        </div>
        {showFooter && (
          <div class="modal-footer">
            {footerActions.map((action, index) => (
              <button
                key={index}
                class={`btn ${variantClasses[action.variant || 'secondary']} ${action.disabled ? 'opacity-50 cursor-not-allowed' : ''}`}
                onClick$={action.onClick$}
                disabled={action.disabled}
              >
                {action.label}
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
});
