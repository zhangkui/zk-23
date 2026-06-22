import { component$ } from '@builder.io/qwik';

interface StatusBadgeProps {
  status: 'online' | 'offline' | 'warning' | 'danger';
  label?: string;
  showDot?: boolean;
}

export default component$<StatusBadgeProps>(({
  status,
  label,
  showDot = true,
}) => {
  const statusConfig = {
    online: {
      dotClass: 'status-online',
      badgeClass: 'badge-success',
      label: '正常',
    },
    offline: {
      dotClass: 'status-offline',
      badgeClass: 'badge-secondary',
      label: '离线',
    },
    warning: {
      dotClass: 'status-warning',
      badgeClass: 'badge-warning',
      label: '告警',
    },
    danger: {
      dotClass: 'status-danger',
      badgeClass: 'badge-danger',
      label: '危险',
    },
  };

  const config = statusConfig[status];
  const displayLabel = label || config.label;

  return (
    <span class={`badge ${config.badgeClass}`}>
      {showDot && <span class={`status-dot ${config.dotClass}`}></span>}
      {displayLabel}
    </span>
  );
});
