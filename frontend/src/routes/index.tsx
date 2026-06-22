import { component$, useStore, useTask$, useVisibleTask$ } from '@builder.io/qwik';
import { Link } from '@builder.io/qwik-city';
import type { Tower, Alert, IceAnalysisResult, WeatherData } from '~/types';
import apiService from '~/services/api';
import websocketService from '~/services/websocket';

export default component$(() => {
  const state = useStore({
    towers: [] as Tower[],
    recentAlerts: [] as Alert[],
    iceAnalysis: [] as IceAnalysisResult[],
    weatherData: null as WeatherData | null,
    alertSummary: { active: 0, acknowledged: 0, resolved: 0 },
    isLoading: true,
    error: '',
  });

  useTask$(async () => {
    state.isLoading = true;
    try {
      const [towersRes, alertsRes, summaryRes] = await Promise.all([
        apiService.getTowers(1, 5),
        apiService.getAlerts(undefined, 'active', undefined, 1, 5),
        apiService.getAlertSummary(),
      ]);

      state.towers = towersRes.data;
      state.recentAlerts = alertsRes.data;
      state.alertSummary = summaryRes;

      if (towersRes.data.length > 0) {
        const firstTowerId = towersRes.data[0].id;
        const [iceRes, weatherRes] = await Promise.all([
          apiService.getLatestIceAnalysis(firstTowerId).catch(() => null),
          apiService.getWeatherData(
            firstTowerId,
            new Date(Date.now() - 3600000).toISOString(),
            new Date().toISOString()
          ).catch(() => []),
        ]);

        if (iceRes) {
          state.iceAnalysis = [iceRes];
        }
        if (weatherRes.length > 0) {
          state.weatherData = weatherRes[weatherRes.length - 1];
        }
      }
    } catch (error) {
      state.error = error instanceof Error ? error.message : '加载数据失败';
    } finally {
      state.isLoading = false;
    }
  });

  useVisibleTask$(() => {
    const unsubscribe = websocketService.onAlert((alert) => {
      state.recentAlerts = [alert, ...state.recentAlerts.slice(0, 4)];
      state.alertSummary.active += 1;
    });

    return () => unsubscribe();
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

  const getRiskLevelColor = (level: string) => {
    const colors: Record<string, string> = {
      none: 'bg-green-100 text-green-800',
      low: 'bg-green-100 text-green-800',
      medium: 'bg-yellow-100 text-yellow-800',
      high: 'bg-orange-100 text-orange-800',
      critical: 'bg-red-100 text-red-800',
      extreme: 'bg-red-200 text-red-900',
    };
    return colors[level] || 'bg-slate-100 text-slate-800';
  };

  const getRiskLevelText = (level: string) => {
    const texts: Record<string, string> = {
      none: '无风险',
      low: '低风险',
      medium: '中风险',
      high: '高风险',
      critical: '严重',
      extreme: '极端',
    };
    return texts[level] || level;
  };

  if (state.isLoading) {
    return (
      <div class="flex items-center justify-center h-64">
        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  return (
    <div class="space-y-6">
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <div class="bg-white rounded-xl shadow-sm p-6 border border-slate-200">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm font-medium text-slate-500">塔架总数</p>
              <p class="text-3xl font-bold text-slate-800 mt-1">{state.towers.length}</p>
            </div>
            <div class="w-12 h-12 bg-blue-100 rounded-lg flex items-center justify-center text-2xl">
              🏗️
            </div>
          </div>
          <div class="mt-4 flex items-center">
            <span class="text-green-600 text-sm">
              {state.towers.filter((t) => t.status === 'normal').length} 个正常
            </span>
            <span class="text-slate-300 mx-2">|</span>
            <span class="text-yellow-600 text-sm">
              {state.towers.filter((t) => t.status === 'warning').length} 个告警
            </span>
          </div>
        </div>

        <div class="bg-white rounded-xl shadow-sm p-6 border border-slate-200">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm font-medium text-slate-500">活动告警</p>
              <p class="text-3xl font-bold text-red-600 mt-1">{state.alertSummary.active}</p>
            </div>
            <div class="w-12 h-12 bg-red-100 rounded-lg flex items-center justify-center text-2xl">
              ⚠️
            </div>
          </div>
          <div class="mt-4 flex items-center">
            <span class="text-yellow-600 text-sm">
              {state.alertSummary.acknowledged} 个已确认
            </span>
            <span class="text-slate-300 mx-2">|</span>
            <span class="text-green-600 text-sm">
              {state.alertSummary.resolved} 个已解决
            </span>
          </div>
        </div>

        <div class="bg-white rounded-xl shadow-sm p-6 border border-slate-200">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm font-medium text-slate-500">最大覆冰厚度</p>
              <p class="text-3xl font-bold text-cyan-600 mt-1">
                {state.iceAnalysis.length > 0
                  ? `${state.iceAnalysis[0].maxIceThicknessMm.toFixed(1)} mm`
                  : '--'}
              </p>
            </div>
            <div class="w-12 h-12 bg-cyan-100 rounded-lg flex items-center justify-center text-2xl">
              ❄️
            </div>
          </div>
          <div class="mt-4">
            {state.iceAnalysis.length > 0 && (
              <span
                class={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getRiskLevelColor(
                  state.iceAnalysis[0].riskLevel
                )}`}
              >
                {getRiskLevelText(state.iceAnalysis[0].riskLevel)}
              </span>
            )}
          </div>
        </div>

        <div class="bg-white rounded-xl shadow-sm p-6 border border-slate-200">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm font-medium text-slate-500">当前风速</p>
              <p class="text-3xl font-bold text-slate-800 mt-1">
                {state.weatherData ? `${state.weatherData.wind_speed_ms.toFixed(1)} m/s` : '--'}
              </p>
            </div>
            <div class="w-12 h-12 bg-sky-100 rounded-lg flex items-center justify-center text-2xl">
              💨
            </div>
          </div>
          <div class="mt-4 flex items-center text-sm text-slate-500">
            <span>温度: {state.weatherData ? `${state.weatherData.temperature_c.toFixed(1)}°C` : '--'}</span>
          </div>
        </div>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div class="bg-white rounded-xl shadow-sm border border-slate-200 overflow-hidden">
          <div class="px-6 py-4 border-b border-slate-200 flex items-center justify-between">
            <h3 class="text-lg font-semibold text-slate-800">塔架状态概览</h3>
            <Link
              href="/towers"
              class="text-sm text-blue-600 hover:text-blue-700"
            >
              查看全部 →
            </Link>
          </div>
          <div class="divide-y divide-slate-100">
            {state.towers.map((tower) => (
              <div key={tower.id} class="px-6 py-4 flex items-center justify-between hover:bg-slate-50">
                <div class="flex items-center">
                  <div class="w-10 h-10 bg-slate-100 rounded-lg flex items-center justify-center text-xl">
                    🏗️
                  </div>
                  <div class="ml-4">
                    <p class="font-medium text-slate-800">{tower.name}</p>
                    <p class="text-sm text-slate-500">{tower.code}</p>
                  </div>
                </div>
                <div class="flex items-center space-x-3">
                  <span
                    class={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getStatusColor(
                      tower.status
                    )}`}
                  >
                    {getStatusText(tower.status)}
                  </span>
                  <Link
                    href={`/towers/${tower.id}`}
                    class="text-slate-400 hover:text-blue-600"
                  >
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                      <path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd" />
                    </svg>
                  </Link>
                </div>
              </div>
            ))}
            {state.towers.length === 0 && (
              <div class="px-6 py-8 text-center text-slate-500">
                暂无塔架数据
              </div>
            )}
          </div>
        </div>

        <div class="bg-white rounded-xl shadow-sm border border-slate-200 overflow-hidden">
          <div class="px-6 py-4 border-b border-slate-200 flex items-center justify-between">
            <h3 class="text-lg font-semibold text-slate-800">最新告警</h3>
            <Link
              href="/alerts"
              class="text-sm text-blue-600 hover:text-blue-700"
            >
              查看全部 →
            </Link>
          </div>
          <div class="divide-y divide-slate-100">
            {state.recentAlerts.map((alert) => (
              <div key={alert.id} class="px-6 py-4 hover:bg-slate-50">
                <div class="flex items-start justify-between">
                  <div class="flex-1">
                    <div class="flex items-center">
                      <span
                        class={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium mr-2 ${getSeverityColor(
                          alert.severity
                        )}`}
                      >
                        {getSeverityText(alert.severity)}
                      </span>
                      <p class="font-medium text-slate-800">{alert.title}</p>
                    </div>
                    <p class="text-sm text-slate-500 mt-1">{alert.message}</p>
                    <p class="text-xs text-slate-400 mt-1">
                      {new Date(alert.triggeredAt).toLocaleString('zh-CN')}
                    </p>
                  </div>
                  <Link
                    href={`/alerts/${alert.id}`}
                    class="text-slate-400 hover:text-blue-600 ml-4"
                  >
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                      <path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd" />
                    </svg>
                  </Link>
                </div>
              </div>
            ))}
            {state.recentAlerts.length === 0 && (
              <div class="px-6 py-8 text-center text-slate-500">
                暂无告警信息
              </div>
            )}
          </div>
        </div>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div class="lg:col-span-2 bg-white rounded-xl shadow-sm border border-slate-200 overflow-hidden">
          <div class="px-6 py-4 border-b border-slate-200">
            <h3 class="text-lg font-semibold text-slate-800">覆冰风险趋势</h3>
          </div>
          <div class="p-6">
            <div class="h-64 flex items-center justify-center text-slate-400">
              <div class="text-center">
                <div class="text-4xl mb-2">📈</div>
                <p>图表组件 - 覆冰厚度变化趋势</p>
                <p class="text-sm mt-1">最大覆冰: {state.iceAnalysis[0]?.maxIceThicknessMm.toFixed(1) || '--'} mm</p>
              </div>
            </div>
          </div>
        </div>

        <div class="bg-white rounded-xl shadow-sm border border-slate-200 overflow-hidden">
          <div class="px-6 py-4 border-b border-slate-200">
            <h3 class="text-lg font-semibold text-slate-800">快速操作</h3>
          </div>
          <div class="p-6 space-y-3">
            <Link
              href="/towers"
              class="flex items-center p-3 rounded-lg bg-blue-50 hover:bg-blue-100 transition-colors"
            >
              <span class="text-xl mr-3">🏗️</span>
              <div>
                <p class="font-medium text-blue-800">塔架管理</p>
                <p class="text-xs text-blue-600">查看和管理所有塔架</p>
              </div>
            </Link>
            <Link
              href="/ice-risk"
              class="flex items-center p-3 rounded-lg bg-cyan-50 hover:bg-cyan-100 transition-colors"
            >
              <span class="text-xl mr-3">❄️</span>
              <div>
                <p class="font-medium text-cyan-800">覆冰风险分析</p>
                <p class="text-xs text-cyan-600">查看覆冰风险评估</p>
              </div>
            </Link>
            <Link
              href="/shutdown"
              class="flex items-center p-3 rounded-lg bg-orange-50 hover:bg-orange-100 transition-colors"
            >
              <span class="text-xl mr-3">⚡</span>
              <div>
                <p class="font-medium text-orange-800">停运策略</p>
                <p class="text-xs text-orange-600">管理停运决策方案</p>
              </div>
            </Link>
            <Link
              href="/video"
              class="flex items-center p-3 rounded-lg bg-purple-50 hover:bg-purple-100 transition-colors"
            >
              <span class="text-xl mr-3">📹</span>
              <div>
                <p class="font-medium text-purple-800">视频复核</p>
                <p class="text-xs text-purple-600">查看摄像头和复核记录</p>
              </div>
            </Link>
          </div>
        </div>
      </div>
    </div>
  );
});
