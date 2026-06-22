import { component$, useStore, useTask$, $ } from '@builder.io/qwik';
import { Link } from '@builder.io/qwik-city';
import type { ShutdownStrategy, Tower } from '~/types';
import apiService from '~/services/api';
import { canTriggerShutdown } from '~/stores/auth';
import { AUTH_CONTEXT } from '~/stores/auth';
import { useContext } from '@builder.io/qwik';

export default component$(() => {
  const auth = useContext(AUTH_CONTEXT);

  const state = useStore({
    towers: [] as Tower[],
    strategies: [] as ShutdownStrategy[],
    total: 0,
    page: 1,
    pageSize: 20,
    isLoading: true,
    error: '',
    statusFilter: '',
    severityFilter: '',
    typeFilter: '',
  });

  const loadData = $(async () => {
    state.isLoading = true;
    try {
      const [towersRes, strategiesRes] = await Promise.all([
        apiService.getTowers(1, 10),
        apiService.getShutdownStrategies(
          undefined,
          state.statusFilter || undefined,
          state.page,
          state.pageSize
        ),
      ]);
      state.towers = towersRes.data;
      state.strategies = strategiesRes.data;
      state.total = strategiesRes.total;
    } catch (error) {
      state.error = error instanceof Error ? error.message : '加载数据失败';
    } finally {
      state.isLoading = false;
    }
  });

  useTask$(() => {
    loadData();
  });

  const handleEvaluate = $(async (towerId: string) => {
    try {
      state.isLoading = true;
      await apiService.evaluateStrategies(towerId);
      loadData();
    } catch (error) {
      console.error('评估失败:', error);
    } finally {
      state.isLoading = false;
    }
  });

  const handleTrigger = $(async (strategyId: string) => {
    if (!confirm('确定要触发此停运策略吗？此操作将执行停运流程。')) return;
    try {
      state.isLoading = true;
      await apiService.triggerShutdown(strategyId);
      loadData();
    } catch (error) {
      console.error('触发停运失败:', error);
    } finally {
      state.isLoading = false;
    }
  });

  const getSeverityColor = (severity: string) => {
    const colors: Record<string, string> = {
      advisory: 'bg-blue-100 text-blue-800',
      watch: 'bg-yellow-100 text-yellow-800',
      warning: 'bg-orange-100 text-orange-800',
      severe: 'bg-red-100 text-red-800',
      extreme: 'bg-red-200 text-red-900',
    };
    return colors[severity] || 'bg-slate-100 text-slate-800';
  };

  const getSeverityText = (severity: string) => {
    const texts: Record<string, string> = {
      advisory: '建议',
      watch: '注意',
      warning: '警告',
      severe: '严重',
      extreme: '极端',
    };
    return texts[severity] || severity;
  };

  const getStatusColor = (status: string) => {
    const colors: Record<string, string> = {
      draft: 'bg-slate-100 text-slate-800',
      pending_approval: 'bg-yellow-100 text-yellow-800',
      approved: 'bg-blue-100 text-blue-800',
      triggered: 'bg-orange-100 text-orange-800',
      executing: 'bg-red-100 text-red-800',
      completed: 'bg-green-100 text-green-800',
      cancelled: 'bg-slate-100 text-slate-600',
      expired: 'bg-slate-100 text-slate-500',
    };
    return colors[status] || 'bg-slate-100 text-slate-800';
  };

  const getStatusText = (status: string) => {
    const texts: Record<string, string> = {
      draft: '草稿',
      pending_approval: '待审批',
      approved: '已批准',
      triggered: '已触发',
      executing: '执行中',
      completed: '已完成',
      cancelled: '已取消',
      expired: '已过期',
    };
    return texts[status] || status;
  };

  const getTypeText = (type: string) => {
    const texts: Record<string, string> = {
      preemptive: '预防性',
      emergency: '紧急',
      scheduled: '计划性',
      manual: '手动',
      recovery: '恢复',
    };
    return texts[type] || type;
  };

  const totalPages = Math.ceil(state.total / state.pageSize);

  return (
    <div class="space-y-6">
      <div class="bg-white rounded-xl shadow-sm border border-slate-200 p-6">
        <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
          <div>
            <h2 class="text-2xl font-bold text-slate-800">停运策略管理</h2>
            <p class="text-slate-500 mt-1">管理和执行索道停运决策方案</p>
          </div>
          <div class="flex items-center gap-4">
            {state.towers.length > 0 && (
              <select
                onChange$={(e) => handleEvaluate((e.target as HTMLSelectElement).value)}
                class="px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
              >
                <option value="">评估塔架策略...</option>
                {state.towers.map((tower) => (
                  <option key={tower.id} value={tower.id}>
                    评估 {tower.name}
                  </option>
                ))}
              </select>
            )}
            {canTriggerShutdown(auth.user) && (
              <button class="inline-flex items-center px-4 py-2 bg-orange-600 text-white rounded-lg hover:bg-orange-700 transition-colors">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 mr-2" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8 7a1 1 0 00-1 1v4a1 1 0 001 1h4a1 1 0 001-1V8a1 1 0 00-1-1H8z" clip-rule="evenodd" />
                </svg>
                创建策略
              </button>
            )}
          </div>
        </div>

        <div class="mt-6 grid grid-cols-1 md:grid-cols-3 gap-4">
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
            <option value="approved">已批准</option>
            <option value="triggered">已触发</option>
            <option value="executing">执行中</option>
            <option value="completed">已完成</option>
            <option value="pending_approval">待审批</option>
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
            <option value="advisory">建议</option>
            <option value="watch">注意</option>
            <option value="warning">警告</option>
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
            <option value="preemptive">预防性</option>
            <option value="emergency">紧急</option>
            <option value="scheduled">计划性</option>
            <option value="manual">手动</option>
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
            <div class="divide-y divide-slate-100">
              {state.strategies.map((strategy) => (
                <div key={strategy.id} class="p-6 hover:bg-slate-50 transition-colors">
                  <div class="flex items-start justify-between">
                    <div class="flex-1">
                      <div class="flex items-center gap-3">
                        <h3 class="text-lg font-semibold text-slate-800">{strategy.title}</h3>
                        <span
                          class={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getSeverityColor(
                            strategy.severity
                          )}`}
                        >
                          {getSeverityText(strategy.severity)}
                        </span>
                        <span
                          class={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getStatusColor(
                            strategy.status
                          )}`}
                        >
                          {getStatusText(strategy.status)}
                        </span>
                        <span class="text-xs text-slate-500">
                          {getTypeText(strategy.strategyType)}
                        </span>
                      </div>
                      <p class="text-sm text-slate-600 mt-2">{strategy.description}</p>
                      <div class="flex items-center gap-6 mt-3 text-sm text-slate-500">
                        <span>预计时长: {strategy.estimatedDurationMinutes} 分钟</span>
                        <span>影响区域: {strategy.affectedArea}</span>
                        <span>触发条件: {strategy.triggerConditions.length} 个</span>
                        <span>行动步骤: {strategy.actionSteps.length} 个</span>
                        {strategy.triggeredAt && (
                          <span>
                            触发时间: {new Date(strategy.triggeredAt).toLocaleString('zh-CN')}
                          </span>
                        )}
                      </div>
                      {strategy.safetyMeasures.length > 0 && (
                        <div class="mt-3 flex flex-wrap gap-2">
                          {strategy.safetyMeasures.slice(0, 5).map((measure, i) => (
                            <span
                              key={i}
                              class="inline-flex items-center px-2 py-0.5 rounded text-xs bg-yellow-50 text-yellow-700"
                            >
                              {measure}
                            </span>
                          ))}
                        </div>
                      )}
                    </div>
                    <div class="flex items-center gap-2 ml-6">
                      {canTriggerShutdown(auth.user) &&
                        (strategy.status === 'approved' || strategy.status === 'pending_approval') && (
                          <button
                            onClick$={() => handleTrigger(strategy.id)}
                            class="px-3 py-1.5 text-sm bg-red-600 text-white rounded-md hover:bg-red-700 transition-colors"
                          >
                            触发停运
                          </button>
                        )}
                      <Link
                        href={`/shutdown/${strategy.id}`}
                        class="px-3 py-1.5 text-sm text-blue-600 hover:bg-blue-50 rounded-md transition-colors"
                      >
                        详情
                      </Link>
                    </div>
                  </div>
                </div>
              ))}
              {state.strategies.length === 0 && (
                <div class="px-6 py-12 text-center text-slate-500">
                  暂无停运策略
                </div>
              )}
            </div>

            <div class="px-6 py-4 border-t border-slate-200 flex items-center justify-between">
              <p class="text-sm text-slate-500">
                显示 {state.strategies.length} / {state.total} 条记录
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
