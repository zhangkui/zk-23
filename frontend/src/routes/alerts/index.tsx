import { component$, useStore, useTask$, $, useVisibleTask$ } from '@builder.io/qwik';
import { Link } from '@builder.io/qwik-city';
import type { Alert } from '~/types';
import apiService from '~/services/api';
import websocketService from '~/services/websocket';
import { canAcknowledgeAlerts } from '~/stores/auth';
import { AUTH_CONTEXT } from '~/stores/auth';
import { useContext } from '@builder.io/qwik';

export default component$(() => {
  const auth = useContext(AUTH_CONTEXT);

  const state = useStore({
    alerts: [] as Alert[],
    total: 0,
    page: 1,
    pageSize: 20,
    isLoading: true,
    error: '',
    statusFilter: 'active',
    severityFilter: '',
    typeFilter: '',
    towerFilter: '',
  });

  const loadData = $(async () => {
    state.isLoading = true;
    try {
      const response = await apiService.getAlerts(
        state.towerFilter || undefined,
        state.statusFilter || undefined,
        state.severityFilter || undefined,
        state.page,
        state.pageSize
      );
      state.alerts = response.data;
      state.total = response.total;
    } catch (error) {
      state.error = error instanceof Error ? error.message : '加载数据失败';
    } finally {
      state.isLoading = false;
    }
  });

  useTask$(() => {
    loadData();
  });

  useVisibleTask$(() => {
    const unsubscribe = websocketService.onAlert((alert) => {
      if (state.statusFilter === 'active') {
        state.alerts = [alert, ...state.alerts.slice(0, state.pageSize - 1)];
        state.total += 1;
      }
    });

    return () => unsubscribe();
  });

  const handleAcknowledge = $(async (alertId: string) => {
    try {
      await apiService.acknowledgeAlert(alertId);
      loadData();
    } catch (error) {
      console.error('确认告警失败:', error);
    }
  });

  const handleResolve = $(async (alertId: string) => {
    const notes = prompt('请输入解决说明:');
    if (notes === null) return;

    try {
      await apiService.resolveAlert(alertId, notes);
      loadData();
    } catch (error) {
      console.error('解决告警失败:', error);
    }
  });

  const getSeverityColor = (severity: string) => {
    const colors: Record<string, string> = {
      low: 'bg-green-100 text-green-800',
      minor: 'bg-blue-100 text-blue-800',
      moderate: 'bg-yellow-100 text-yellow-800',
      severe: 'bg-orange-100 text-orange-800',
      extreme: 'bg-red-100 text-red-800',
    };
    return colors[severity] || 'bg-slate-100 text-slate-800';
  };

  const getSeverityText = (severity: string) => {
    const texts: Record<string, string> = {
      low: '低',
      minor: '轻微',
      moderate: '中等',
      severe: '严重',
      extreme: '极端',
    };
    return texts[severity] || severity;
  };

  const getStatusColor = (status: string) => {
    const colors: Record<string, string> = {
      active: 'bg-red-100 text-red-800',
      acknowledged: 'bg-yellow-100 text-yellow-800',
      resolved: 'bg-green-100 text-green-800',
      expired: 'bg-slate-100 text-slate-800',
    };
    return colors[status] || 'bg-slate-100 text-slate-800';
  };

  const getStatusText = (status: string) => {
    const texts: Record<string, string> = {
      active: '活动',
      acknowledged: '已确认',
      resolved: '已解决',
      expired: '已过期',
    };
    return texts[status] || status;
  };

  const getTypeText = (type: string) => {
    const texts: Record<string, string> = {
      vibration: '振动',
      wind_speed: '风速',
      ice_detection: '覆冰',
      temperature: '温度',
      tilt: '倾斜',
      strain: '应变',
      system: '系统',
      video: '视频',
      weather: '天气',
    };
    return texts[type] || type;
  };

  const totalPages = Math.ceil(state.total / state.pageSize);

  return (
    <div class="space-y-6">
      <div class="bg-white rounded-xl shadow-sm border border-slate-200 p-6">
        <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
          <div>
            <h2 class="text-2xl font-bold text-slate-800">告警中心</h2>
            <p class="text-slate-500 mt-1">查看和处理系统告警信息</p>
          </div>
        </div>

        <div class="mt-6 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          <select
            value={state.statusFilter}
            onChange$={(e) => {
              state.statusFilter = (e.target as HTMLSelectElement).value;
              state.page = 1;
              loadData();
            }}
            class="px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
          >
            <option value="">全部状态</option>
            <option value="active">活动</option>
            <option value="acknowledged">已确认</option>
            <option value="resolved">已解决</option>
            <option value="expired">已过期</option>
          </select>

          <select
            value={state.severityFilter}
            onChange$={(e) => {
              state.severityFilter = (e.target as HTMLSelectElement).value;
              state.page = 1;
              loadData();
            }}
            class="px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
          >
            <option value="">全部级别</option>
            <option value="low">低</option>
            <option value="minor">轻微</option>
            <option value="moderate">中等</option>
            <option value="severe">严重</option>
            <option value="extreme">极端</option>
          </select>

          <select
            value={state.typeFilter}
            onChange$={(e) => {
              state.typeFilter = (e.target as HTMLSelectElement).value;
              state.page = 1;
              loadData();
            }}
            class="px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
          >
            <option value="">全部类型</option>
            <option value="vibration">振动</option>
            <option value="wind_speed">风速</option>
            <option value="ice_detection">覆冰</option>
            <option value="temperature">温度</option>
            <option value="weather">天气</option>
            <option value="system">系统</option>
          </select>

          <select
            value={state.towerFilter}
            onChange$={(e) => {
              state.towerFilter = (e.target as HTMLSelectElement).value;
              state.page = 1;
              loadData();
            }}
            class="px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
          >
            <option value="">全部塔架</option>
          </select>
        </div>
      </div>

      <div class="bg-white rounded-xl shadow-sm border border-slate-200 overflow-hidden">
        {state.isLoading ? (
          <div class="flex items-center justify-center h-64">
            <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
          </div>
        ) : state.error ? (
          <div class="p-8 text-center text-red-600">
            {state.error}
          </div>
        ) : (
          <>
            <div class="overflow-x-auto">
              <table class="w-full">
                <thead class="bg-slate-50 border-b border-slate-200">
                  <tr>
                    <th class="px-6 py-4 text-left text-sm font-semibold text-slate-800">级别</th>
                    <th class="px-6 py-4 text-left text-sm font-semibold text-slate-800">标题</th>
                    <th class="px-6 py-4 text-left text-sm font-semibold text-slate-800">类型</th>
                    <th class="px-6 py-4 text-left text-sm font-semibold text-slate-800">状态</th>
                    <th class="px-6 py-4 text-left text-sm font-semibold text-slate-800">触发时间</th>
                    <th class="px-6 py-4 text-right text-sm font-semibold text-slate-800">操作</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-slate-100">
                  {state.alerts.map((alert) => (
                    <tr key={alert.id} class="hover:bg-slate-50 transition-colors">
                      <td class="px-6 py-4">
                        <span
                          class={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getSeverityColor(
                            alert.severity
                          )}`}
                        >
                          {getSeverityText(alert.severity)}
                        </span>
                      </td>
                      <td class="px-6 py-4">
                        <div>
                          <p class="font-medium text-slate-800">{alert.title}</p>
                          <p class="text-sm text-slate-500 truncate max-w-md">{alert.message}</p>
                        </div>
                      </td>
                      <td class="px-6 py-4">
                        <span class="text-sm text-slate-600">{getTypeText(alert.type)}</span>
                      </td>
                      <td class="px-6 py-4">
                        <span
                          class={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getStatusColor(
                            alert.status
                          )}`}
                        >
                          {getStatusText(alert.status)}
                        </span>
                      </td>
                      <td class="px-6 py-4">
                        <span class="text-sm text-slate-600">
                          {new Date(alert.triggeredAt).toLocaleString('zh-CN')}
                        </span>
                      </td>
                      <td class="px-6 py-4 text-right">
                        <div class="flex items-center justify-end space-x-2">
                          {canAcknowledgeAlerts(auth.user) && alert.status === 'active' && (
                            <button
                              onClick$={() => handleAcknowledge(alert.id)}
                              class="px-3 py-1 text-sm bg-yellow-100 text-yellow-700 rounded-md hover:bg-yellow-200 transition-colors"
                            >
                              确认
                            </button>
                          )}
                          {canAcknowledgeAlerts(auth.user) && (alert.status === 'active' || alert.status === 'acknowledged') && (
                            <button
                              onClick$={() => handleResolve(alert.id)}
                              class="px-3 py-1 text-sm bg-green-100 text-green-700 rounded-md hover:bg-green-200 transition-colors"
                            >
                              解决
                            </button>
                          )}
                          <Link
                            href={`/alerts/${alert.id}`}
                            class="px-3 py-1 text-sm text-blue-600 hover:bg-blue-50 rounded-md transition-colors"
                          >
                            详情
                          </Link>
                        </div>
                      </td>
                    </tr>
                  ))}
                  {state.alerts.length === 0 && (
                    <tr>
                      <td colspan="6" class="px-6 py-12 text-center text-slate-500">
                        暂无告警信息
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>

            <div class="px-6 py-4 border-t border-slate-200 flex items-center justify-between">
              <p class="text-sm text-slate-500">
                显示 {state.alerts.length} / {state.total} 条记录
              </p>
              <div class="flex items-center space-x-2">
                <button
                  onClick$={() => {
                    if (state.page > 1) {
                      state.page--;
                      loadData();
                    }
                  }}
                  disabled={state.page === 1}
                  class="px-3 py-1 border border-slate-300 rounded-md text-sm disabled:opacity-50 disabled:cursor-not-allowed hover:bg-slate-50"
                >
                  上一页
                </button>
                <span class="text-sm text-slate-600">
                  {state.page} / {totalPages || 1}
                </span>
                <button
                  onClick$={() => {
                    if (state.page < totalPages) {
                      state.page++;
                      loadData();
                    }
                  }}
                  disabled={state.page >= totalPages}
                  class="px-3 py-1 border border-slate-300 rounded-md text-sm disabled:opacity-50 disabled:cursor-not-allowed hover:bg-slate-50"
                >
                  下一页
                </button>
              </div>
            </div>
          </>
        )}
      </div>
    </div>
  );
});
