import { component$, useStore, useTask$, $ } from '@builder.io/qwik';
import { Link } from '@builder.io/qwik-city';
import type { Tower } from '~/types';
import apiService from '~/services/api';
import { canManageTowers } from '~/stores/auth';
import { AUTH_CONTEXT } from '~/stores/auth';
import { useContext } from '@builder.io/qwik';

export default component$(() => {
  const auth = useContext(AUTH_CONTEXT);

  const state = useStore({
    towers: [] as Tower[],
    total: 0,
    page: 1,
    pageSize: 20,
    isLoading: true,
    error: '',
    searchQuery: '',
    statusFilter: '',
  });

  const loadData = $(async () => {
    state.isLoading = true;
    try {
      const response = await apiService.getTowers(state.page, state.pageSize);
      state.towers = response.data;
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

  const getStatusColor = (status: string) => {
    const colors: Record<string, string> = {
      normal: 'bg-green-100 text-green-800',
      warning: 'bg-yellow-100 text-yellow-800',
      danger: 'bg-red-100 text-red-800',
      maintenance: 'bg-blue-100 text-blue-800',
      offline: 'bg-slate-100 text-slate-800',
    };
    return colors[status] || 'bg-slate-100 text-slate-800';
  };

  const getStatusText = (status: string) => {
    const texts: Record<string, string> = {
      normal: '正常',
      warning: '告警',
      danger: '危险',
      maintenance: '维护',
      offline: '离线',
    };
    return texts[status] || status;
  };

  const filteredTowers = state.towers.filter((tower) => {
    const matchesSearch =
      tower.name.toLowerCase().includes(state.searchQuery.toLowerCase()) ||
      tower.code.toLowerCase().includes(state.searchQuery.toLowerCase());
    const matchesStatus = !state.statusFilter || tower.status === state.statusFilter;
    return matchesSearch && matchesStatus;
  });

  const totalPages = Math.ceil(state.total / state.pageSize);

  return (
    <div class="space-y-6">
      <div class="bg-white rounded-xl shadow-sm border border-slate-200 p-6">
        <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
          <div class="flex-1">
            <h2 class="text-2xl font-bold text-slate-800">塔架管理</h2>
            <p class="text-slate-500 mt-1">管理所有索道塔架及其设备配置</p>
          </div>
          {canManageTowers(auth.user) && (
            <button class="inline-flex items-center px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 mr-2" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clip-rule="evenodd" />
              </svg>
              添加塔架
            </button>
          )}
        </div>

        <div class="mt-6 flex flex-col md:flex-row gap-4">
          <div class="flex-1 relative">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              class="h-5 w-5 absolute left-3 top-1/2 transform -translate-y-1/2 text-slate-400"
              viewBox="0 0 20 20"
              fill="currentColor"
            >
              <path fill-rule="evenodd" d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z" clip-rule="evenodd" />
            </svg>
            <input
              type="text"
              placeholder="搜索塔架名称或编号..."
              value={state.searchQuery}
              onInput$={(e) => (state.searchQuery = (e.target as HTMLInputElement).value)}
              class="w-full pl-10 pr-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
            />
          </div>
          <select
            value={state.statusFilter}
            onChange$={(e) => (state.statusFilter = (e.target as HTMLSelectElement).value)}
            class="px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
          >
            <option value="">全部状态</option>
            <option value="normal">正常</option>
            <option value="warning">告警</option>
            <option value="danger">危险</option>
            <option value="maintenance">维护</option>
            <option value="offline">离线</option>
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
                    <th class="px-6 py-4 text-left text-sm font-semibold text-slate-800">塔架</th>
                    <th class="px-6 py-4 text-left text-sm font-semibold text-slate-800">位置</th>
                    <th class="px-6 py-4 text-left text-sm font-semibold text-slate-800">状态</th>
                    <th class="px-6 py-4 text-left text-sm font-semibold text-slate-800">传感器</th>
                    <th class="px-6 py-4 text-left text-sm font-semibold text-slate-800">摄像头</th>
                    <th class="px-6 py-4 text-left text-sm font-semibold text-slate-800">下次巡检</th>
                    <th class="px-6 py-4 text-right text-sm font-semibold text-slate-800">操作</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-slate-100">
                  {filteredTowers.map((tower) => (
                    <tr key={tower.id} class="hover:bg-slate-50 transition-colors">
                      <td class="px-6 py-4">
                        <div class="flex items-center">
                          <div class="w-10 h-10 bg-slate-100 rounded-lg flex items-center justify-center text-xl">
                            🏗️
                          </div>
                          <div class="ml-4">
                            <p class="font-medium text-slate-800">{tower.name}</p>
                            <p class="text-sm text-slate-500">{tower.code}</p>
                          </div>
                        </div>
                      </td>
                      <td class="px-6 py-4">
                        <p class="text-sm text-slate-600">
                          {tower.location.lat.toFixed(4)}, {tower.location.lng.toFixed(4)}
                        </p>
                        <p class="text-xs text-slate-400">海拔 {tower.location.altitude}m</p>
                      </td>
                      <td class="px-6 py-4">
                        <span
                          class={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getStatusColor(
                            tower.status
                          )}`}
                        >
                          {getStatusText(tower.status)}
                        </span>
                      </td>
                      <td class="px-6 py-4">
                        <span class="text-sm text-slate-600">{tower.sensors?.length || 0} 个</span>
                      </td>
                      <td class="px-6 py-4">
                        <span class="text-sm text-slate-600">{tower.cameras?.length || 0} 个</span>
                      </td>
                      <td class="px-6 py-4">
                        <span class="text-sm text-slate-600">
                          {new Date(tower.nextInspectionDate).toLocaleDateString('zh-CN')}
                        </span>
                      </td>
                      <td class="px-6 py-4 text-right">
                        <Link
                          href={`/towers/${tower.id}`}
                          class="inline-flex items-center px-3 py-1.5 text-sm text-blue-600 hover:bg-blue-50 rounded-md transition-colors"
                        >
                          查看详情
                          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 ml-1" viewBox="0 0 20 20" fill="currentColor">
                            <path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd" />
                          </svg>
                        </Link>
                      </td>
                    </tr>
                  ))}
                  {filteredTowers.length === 0 && (
                    <tr>
                      <td colspan="7" class="px-6 py-12 text-center text-slate-500">
                        暂无匹配的塔架数据
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>

            <div class="px-6 py-4 border-t border-slate-200 flex items-center justify-between">
              <p class="text-sm text-slate-500">
                显示 {filteredTowers.length} / {state.total} 条记录
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
