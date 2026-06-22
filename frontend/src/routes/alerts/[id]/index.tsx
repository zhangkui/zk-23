import { component$, useStore, useTask$ } from '@builder.io/qwik';
import { routeLoader$ } from '@builder.io/qwik-city';
import { api } from '~/services/api';
import type { Alert } from '~/types';
import StatusBadge from '~/components/StatusBadge';
import dayjs from 'dayjs';

export const useAlertData = routeLoader$(async ({ params }) => {
  try {
    const alert = await api.alerts.getById(params.id);
    return { alert };
  } catch (error) {
    return { alert: null };
  }
});

const severityLabels: Record<string, { label: string; color: string }> = {
  low: { label: '低', color: 'badge-info' },
  medium: { label: '中', color: 'badge-warning' },
  high: { label: '高', color: 'badge-warning' },
  severe: { label: '严重', color: 'badge-danger' },
  critical: { label: '紧急', color: 'badge-danger' },
};

const statusLabels: Record<string, string> = {
  active: '触发',
  acknowledged: '已确认',
  resolved: '已解决',
  closed: '已关闭',
};

const typeLabels: Record<string, string> = {
  vibration: '振动异常',
  wind_speed: '风速异常',
  ice_detection: '覆冰告警',
  system: '系统告警',
  equipment: '设备故障',
};

export default component$(() => {
  const alertData = useAlertData();
  const state = useStore({
    isProcessing: false,
  });

  const alert = alertData.value.alert as Alert | null;

  if (!alert) {
    return (
      <div class="card p-12 text-center">
        <div class="text-gray-400 text-6xl mb-4">⚠️</div>
        <h3 class="text-xl font-semibold text-gray-700 mb-2">告警不存在</h3>
        <p class="text-gray-500">未找到对应的告警信息</p>
      </div>
    );
  }

  const severityConfig = severityLabels[alert.severity] || severityLabels.low;

  const handleAcknowledge = async () => {
    if (!alert.id) return;
    state.isProcessing = true;
    try {
      await api.alerts.acknowledge(alert.id, { note: '运维人员已确认告警' });
      alert.status = 'acknowledged';
    } catch (error) {
      console.error('确认告警失败:', error);
    } finally {
      state.isProcessing = false;
    }
  };

  const handleResolve = async () => {
    if (!alert.id) return;
    state.isProcessing = true;
    try {
      await api.alerts.resolve(alert.id, { resolution: '问题已修复，系统恢复正常' });
      alert.status = 'resolved';
    } catch (error) {
      console.error('解决告警失败:', error);
    } finally {
      state.isProcessing = false;
    }
  };

  return (
    <div class="space-y-6">
      <div class="flex items-center justify-between">
        <div>
          <div class="flex items-center gap-3 mb-2">
            <h1 class="text-2xl font-bold text-gray-900">{alert.title}</h1>
            <span class={`badge ${severityConfig.color}`}>{severityConfig.label}级</span>
          </div>
          <p class="text-gray-500">
            告警编号: {alert.id} | {typeLabels[alert.type] || alert.type}
          </p>
        </div>
        <button class="btn btn-secondary" onClick$={() => history.back()}>
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M15 19l-7-7 7-7" />
          </svg>
          返回列表
        </button>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div class="lg:col-span-2 space-y-6">
          <div class="card">
            <div class="card-header">告警详情</div>
            <div class="card-body space-y-4">
              <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div>
                  <label class="label">告警状态</label>
                  <div class="flex items-center gap-2">
                    <span class={`badge ${
                      alert.status === 'active' ? 'badge-danger' :
                      alert.status === 'acknowledged' ? 'badge-warning' :
                      'badge-success'
                    }`}>
                      {statusLabels[alert.status] || alert.status}
                    </span>
                  </div>
                </div>
                <div>
                  <label class="label">关联塔架</label>
                  <p class="text-gray-900 font-medium">
                    {alert.tower_name || '未关联'}
                    {alert.tower_id && (
                      <a href={`/towers/${alert.tower_id}`} class="text-primary-600 hover:underline ml-2 text-sm">
                        查看详情 →
                      </a>
                    )}
                  </p>
                </div>
                <div>
                  <label class="label">触发时间</label>
                  <p class="text-gray-900 font-medium">
                    {dayjs(alert.triggered_at).format('YYYY-MM-DD HH:mm:ss')}
                  </p>
                </div>
                <div>
                  <label class="label">确认时间</label>
                  <p class="text-gray-900 font-medium">
                    {alert.acknowledged_at
                      ? dayjs(alert.acknowledged_at).format('YYYY-MM-DD HH:mm:ss')
                      : '未确认'}
                  </p>
                </div>
                <div>
                  <label class="label">解决时间</label>
                  <p class="text-gray-900 font-medium">
                    {alert.resolved_at
                      ? dayjs(alert.resolved_at).format('YYYY-MM-DD HH:mm:ss')
                      : '未解决'}
                  </p>
                </div>
                <div>
                  <label class="label">确认人</label>
                  <p class="text-gray-900 font-medium">
                    {alert.acknowledged_by || '未确认'}
                  </p>
                </div>
              </div>

              <div class="pt-4 border-t border-gray-200">
                <label class="label">告警描述</label>
                <p class="text-gray-700 bg-gray-50 p-4 rounded-lg">
                  {alert.description || '无详细描述'}
                </p>
              </div>

              {alert.threshold_value !== null && alert.actual_value !== null && (
                <div class="p-4 bg-orange-50 border border-orange-200 rounded-lg">
                  <h4 class="font-medium text-orange-800 mb-2">阈值信息</h4>
                  <div class="grid grid-cols-2 gap-4 text-sm">
                    <div>
                      <span class="text-orange-600">阈值:</span>
                      <span class="font-medium text-orange-800 ml-2">{alert.threshold_value}</span>
                    </div>
                    <div>
                      <span class="text-orange-600">实际值:</span>
                      <span class="font-medium text-orange-800 ml-2">{alert.actual_value}</span>
                    </div>
                  </div>
                </div>
              )}

              {alert.resolution && (
                <div class="p-4 bg-green-50 border border-green-200 rounded-lg">
                  <h4 class="font-medium text-green-800 mb-2">解决方案</h4>
                  <p class="text-green-700">{alert.resolution}</p>
                </div>
              )}
            </div>
          </div>

          <div class="card">
            <div class="card-header">操作记录</div>
            <div class="card-body">
              <div class="space-y-4">
                <div class="flex gap-4">
                  <div class="flex flex-col items-center">
                    <div class="w-3 h-3 rounded-full bg-primary-500"></div>
                    <div class="w-0.5 h-full bg-gray-200 mt-1"></div>
                  </div>
                  <div class="flex-1 pb-4">
                    <div class="flex items-center gap-2 mb-1">
                      <span class="font-medium text-gray-900">告警触发</span>
                      <span class="text-xs text-gray-500">
                        {dayjs(alert.triggered_at).format('YYYY-MM-DD HH:mm:ss')}
                      </span>
                    </div>
                    <p class="text-sm text-gray-600">系统自动检测到异常，触发告警</p>
                  </div>
                </div>

                {alert.acknowledged_at && (
                  <div class="flex gap-4">
                    <div class="flex flex-col items-center">
                      <div class="w-3 h-3 rounded-full bg-warning-500"></div>
                      <div class="w-0.5 h-full bg-gray-200 mt-1"></div>
                    </div>
                    <div class="flex-1 pb-4">
                      <div class="flex items-center gap-2 mb-1">
                        <span class="font-medium text-gray-900">告警确认</span>
                        <span class="text-xs text-gray-500">
                          {dayjs(alert.acknowledged_at).format('YYYY-MM-DD HH:mm:ss')}
                        </span>
                      </div>
                      <p class="text-sm text-gray-600">
                        {alert.acknowledged_by || '运维人员'} 已确认此告警
                        {alert.acknowledge_note && `: ${alert.acknowledge_note}`}
                      </p>
                    </div>
                  </div>
                )}

                {alert.resolved_at && (
                  <div class="flex gap-4">
                    <div class="flex flex-col items-center">
                      <div class="w-3 h-3 rounded-full bg-success-500"></div>
                    </div>
                    <div class="flex-1">
                      <div class="flex items-center gap-2 mb-1">
                        <span class="font-medium text-gray-900">告警解决</span>
                        <span class="text-xs text-gray-500">
                          {dayjs(alert.resolved_at).format('YYYY-MM-DD HH:mm:ss')}
                        </span>
                      </div>
                      <p class="text-sm text-gray-600">
                        {alert.resolved_by || '运维人员'} 已标记此告警为已解决
                        {alert.resolution && `: ${alert.resolution}`}
                      </p>
                    </div>
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>

        <div class="space-y-6">
          <div class="card">
            <div class="card-header">快捷操作</div>
            <div class="card-body space-y-3">
              {alert.status === 'active' && (
                <>
                  <button
                    class="btn btn-warning w-full justify-center"
                    onClick$={handleAcknowledge}
                    disabled={state.isProcessing}
                  >
                    {state.isProcessing ? '处理中...' : '确认告警'}
                  </button>
                  <button
                    class="btn btn-success w-full justify-center"
                    onClick$={handleResolve}
                    disabled={state.isProcessing}
                  >
                    {state.isProcessing ? '处理中...' : '标记解决'}
                  </button>
                </>
              )}
              {alert.status === 'acknowledged' && (
                <button
                  class="btn btn-success w-full justify-center"
                  onClick$={handleResolve}
                  disabled={state.isProcessing}
                >
                  {state.isProcessing ? '处理中...' : '标记解决'}
                </button>
              )}
              {alert.status === 'resolved' && (
                <div class="alert alert-success text-center">
                  此告警已解决
                </div>
              )}

              {alert.tower_id && (
                <a href={`/towers/${alert.tower_id}`} class="btn btn-secondary w-full justify-center">
                  查看关联塔架
                </a>
              )}

              <a href={`/video?tower_id=${alert.tower_id}`} class="btn btn-secondary w-full justify-center">
                视频复核
              </a>

              <a href={`/inspections?new=true&tower_id=${alert.tower_id}`} class="btn btn-secondary w-full justify-center">
                创建巡检任务
              </a>
            </div>
          </div>

          <div class="card">
            <div class="card-header">告警建议</div>
            <div class="card-body">
              <div class="space-y-3">
                {alert.severity === 'critical' || alert.severity === 'severe' ? (
                  <div class="alert alert-danger text-sm">
                    <strong>紧急处理:</strong> 此告警级别较高，建议立即安排人员现场检查，必要时启动停运程序。
                  </div>
                ) : alert.severity === 'high' || alert.severity === 'medium' ? (
                  <div class="alert alert-warning text-sm">
                    <strong>及时处理:</strong> 请尽快安排运维人员检查相关设备，确认告警原因。
                  </div>
                ) : (
                  <div class="alert alert-info text-sm">
                    <strong>常规处理:</strong> 请在日常巡检中关注此问题，必要时进行排查。
                  </div>
                )}

                <div class="p-3 bg-gray-50 rounded-lg text-sm">
                  <h4 class="font-medium text-gray-700 mb-2">可能的原因</h4>
                  <ul class="text-gray-600 space-y-1 list-disc list-inside">
                    <li>传感器校准偏移</li>
                    <li>环境条件突变</li>
                    <li>设备老化或损坏</li>
                    <li>安装松动</li>
                  </ul>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
});
