import { component$, useStore, useTask$, $ } from '@builder.io/qwik';
import { Link } from '@builder.io/qwik-city';
import type { InspectionRecord, Tower } from '~/types';
import apiService from '~/services/api';
import { canCreateInspections } from '~/stores/auth';
import { AUTH_CONTEXT } from '~/stores/auth';
import { useContext } from '@builder.io/qwik';

export default component$(() => {
  const auth = useContext(AUTH_CONTEXT);

  const state = useStore({
    towers: [] as Tower[],
    inspections: [] as InspectionRecord[],
    total: 0,
    page: 1,
    pageSize: 20,
    isLoading: true,
    error: '',
    typeFilter: '',
    statusFilter: '',
    towerFilter: '',
  });

  const loadData = $(async () => {
    state.isLoading = true;
    try {
      const [towersRes, inspectionsRes] = await Promise.all([
        apiService.getTowers(1, 10),
        apiService.getInspections(
          state.towerFilter || undefined,
          state.typeFilter || undefined,
          state.statusFilter || undefined,
          state.page,
          state.pageSize
        ),
      ]);
      state.towers = towersRes.data;
      state.inspections = inspectionsRes.data;
      state.total = inspectionsRes.total;
    } catch (error) {
      state.error = error instanceof Error ? error.message : '加载数据失败';
    } finally {
      state.isLoading = false;
    }
  });

  useTask$(() => {
    loadData();
  });

  const handleDownloadReport = $(async (inspectionId: string) => {
    try {
      const blob = await apiService.generateInspectionReport(inspectionId);
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `inspection-report-${inspectionId}.pdf`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      window.URL.revokeObjectURL(url);
    } catch (error) {
      console.error('下载报告失败:', error);
    }
  });

  const getTypeText = (type: string) => {
    const texts: Record<string, string> = {
      routine: '例行巡检',
      comprehensive: '全面检查',
      post_incident: '事故后检查',
      post_storm: '风暴后检查',
      emergency: '紧急检查',
      specialized: '专项检查',
    };
    return texts[type] || type;
  };

  const getTypeColor = (type: string) => {
    const colors: Record<string, string> = {
      routine: 'bg-blue-100 text-blue-800',
      comprehensive: 'bg-purple-100 text-purple-800',
      post_incident: 'bg-red-100 text-red-800',
      post_storm: 'bg-orange-100 text-orange-800',
      emergency: 'bg-red-200 text-red-900',
      specialized: 'bg-cyan-100 text-cyan-800',
    };
    return colors[type] || 'bg-slate-100 text-slate-800';
  };

  const getStatusText = (status: string) => {
    const texts: Record<string, string> = {
      scheduled: '已计划',
      in_progress: '进行中',
      completed: '已完成',
      cancelled: '已取消',
      overdue: '已逾期',
    };
    return texts[status] || status;
  };

  const getStatusColor = (status: string) => {
    const colors: Record<string, string> = {
      scheduled: 'bg-yellow-100 text-yellow-800',
      in_progress: 'bg-blue-100 text-blue-800',
      completed: 'bg-green-100 text-green-800',
      cancelled: 'bg-slate-100 text-slate-600',
      overdue: 'bg-red-100 text-red-800',
    };
    return colors[status] || 'bg-slate-100 text-slate-800';
  };

  const getConditionText = (condition: string) => {
    const texts: Record<string, string> = {
      excellent: '优秀',
      good: '良好',
      fair: '一般',
      poor: '较差',
      critical: '危险',
      safety: '安全隐患',
    };
    return texts[condition] || condition;
  };

  const getConditionColor = (condition: string) => {
    const colors: Record<string, string> = {
      excellent: 'bg-green-100 text-green-800',
      good: 'bg-blue-100 text-blue-800',
      fair: 'bg-yellow-100 text-yellow-800',
      poor: 'bg-orange-100 text-orange-800',
      critical: 'bg-red-100 text-red-800',
      safety: 'bg-red-200 text-red-900',
    };
    return colors[condition] || 'bg-slate-100 text-slate-800';
  };

  const getTowerName = (towerId: string) => {
    const tower = state.towers.find((t) => t.id === towerId);
    return tower?.name || towerId;
  };

  const totalPages = Math.ceil(state.total / state.pageSize);

  return (
    <div class="space-y-6">
      <div class="bg-white rounded-xl shadow-sm border border-slate-200 p-6">
        <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
          <div>
            <h2 class="text-2xl font-bold text-slate-800">巡检记录</h2>
            <p class="text-slate-500 mt-1">管理和查看塔架巡检记录</p>
          </div>
          {canCreateInspections(auth.user) && (
            <button class="inline-flex items-center px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 mr-2" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clip-rule="evenodd" />
              </svg>
              创建巡检
            </button>
          )}
        </div>

        <div class="mt-6 grid grid-cols-1 md:grid-cols-3 gap-4">
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
            {state.towers.map((tower) => (
              <option key={tower.id} value={tower.id}>
                {tower.name}
              </option>
            ))}
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
            <option value="routine">例行巡检</option>
            <option value="comprehensive">全面检查</option>
            <option value="post_storm">风暴后检查</option>
            <option value="emergency">紧急检查</option>
          </select>

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
            <option value="scheduled">已计划</option>
            <option value="in_progress">进行中</option>
            <option value="completed">已完成</option>
            <option value="overdue">已逾期</option>
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
              {state.inspections.map((inspection) => (
                <div key={inspection.id} class="p-6 hover:bg-slate-50 transition-colors">
                  <div class="flex items-start justify-between">
                    <div class="flex-1">
                      <div class="flex items-center gap-3">
                        <h3 class="text-lg font-semibold text-slate-800">
                          {getTypeText(inspection.type)} - {getTowerName(inspection.towerId)}
                        </h3>
                        <span
                          class={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getTypeColor(
                            inspection.type
                          )}`}
                        >
                          {getTypeText(inspection.type)}
                        </span>
                        <span
                          class={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getStatusColor(
                            inspection.status
                          )}`}
                        >
                          {getStatusText(inspection.status)}
                        </span>
                        {inspection.overallCondition && (
                          <span
                            class={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getConditionColor(
                              inspection.overallCondition
                            )}`}
                          >
                            {getConditionText(inspection.overallCondition)}
                          </span>
                        )}
                      </div>
                      <div class="flex items-center gap-6 mt-3 text-sm text-slate-500">
                        <span>
                          巡检日期: {new Date(inspection.inspectionDate).toLocaleDateString('zh-CN')}
                        </span>
                        <span>
                          开始时间: {new Date(inspection.startTime).toLocaleTimeString('zh-CN')}
                        </span>
                        {inspection.endTime && (
                          <span>
                            结束时间: {new Date(inspection.endTime).toLocaleTimeString('zh-CN')}
                          </span>
                        )}
                        {inspection.weatherConditions && (
                          <span>天气: {inspection.weatherConditions}</span>
                        )}
                      </div>
                      <div class="flex items-center gap-6 mt-2 text-sm text-slate-500">
                        <span>发现问题: {inspection.findings.length} 个</span>
                        <span>维护任务: {inspection.maintenanceTasks.length} 个</span>
                        <span>建议: {inspection.recommendations.length} 条</span>
                      </div>
                      {inspection.findings.length > 0 && (
                        <div class="mt-3 flex flex-wrap gap-2">
                          {inspection.findings.slice(0, 5).map((finding, i) => (
                            <span
                              key={i}
                              class="inline-flex items-center px-2 py-0.5 rounded text-xs bg-red-50 text-red-700"
                            >
                              {finding.category}: {finding.description.slice(0, 30)}...
                            </span>
                          ))}
                        </div>
                      )}
                    </div>
                    <div class="flex items-center gap-2 ml-6">
                      {inspection.status === 'completed' && (
                        <button
                          onClick$={() => handleDownloadReport(inspection.id)}
                          class="px-3 py-1.5 text-sm bg-green-100 text-green-700 rounded-md hover:bg-green-200 transition-colors"
                        >
                          下载报告
                        </button>
                      )}
                      <Link
                        href={`/inspections/${inspection.id}`}
                        class="px-3 py-1.5 text-sm text-blue-600 hover:bg-blue-50 rounded-md transition-colors"
                      >
                        详情
                      </Link>
                    </div>
                  </div>
                </div>
              ))}
              {state.inspections.length === 0 && (
                <div class="px-6 py-12 text-center text-slate-500">
                  暂无巡检记录
                </div>
              )}
            </div>

            <div class="px-6 py-4 border-t border-slate-200 flex items-center justify-between">
              <p class="text-sm text-slate-500">
                显示 {state.inspections.length} / {state.total} 条记录
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
