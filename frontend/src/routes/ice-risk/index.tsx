import { component$, useStore, useTask$, $ } from '@builder.io/qwik';
import type { IceAnalysisResult, Tower } from '~/types';
import apiService from '~/services/api';

export default component$(() => {
  const state = useStore({
    towers: [] as Tower[],
    selectedTowerId: '',
    analysisResults: [] as IceAnalysisResult[],
    latestAnalysis: null as IceAnalysisResult | null,
    isLoading: true,
    error: '',
  });

  const loadData = $(async () => {
    state.isLoading = true;
    try {
      const towersRes = await apiService.getTowers(1, 10);
      state.towers = towersRes.data;

      if (towersRes.data.length > 0) {
        state.selectedTowerId = state.selectedTowerId || towersRes.data[0].id;
        const [analysisRes, latestRes] = await Promise.all([
          apiService.getIceAnalysis(state.selectedTowerId, 24),
          apiService.getLatestIceAnalysis(state.selectedTowerId).catch(() => null),
        ]);
        state.analysisResults = analysisRes;
        state.latestAnalysis = latestRes;
      }
    } catch (error) {
      state.error = error instanceof Error ? error.message : '加载数据失败';
    } finally {
      state.isLoading = false;
    }
  });

  useTask$(() => {
    loadData();
  });

  const handleTowerChange = $((towerId: string) => {
    state.selectedTowerId = towerId;
    loadData();
  });

  const handleAnalyze = $(async () => {
    if (!state.selectedTowerId) return;
    try {
      state.isLoading = true;
      const result = await apiService.triggerIceAnalysis(state.selectedTowerId);
      state.latestAnalysis = result;
      state.analysisResults = [result, ...state.analysisResults.slice(0, 23)];
    } catch (error) {
      console.error('分析失败:', error);
    } finally {
      state.isLoading = false;
    }
  });

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

  const getIceTypeText = (type: string) => {
    const texts: Record<string, string> = {
      none: '无覆冰',
      rime: '雾凇',
      glaze: '雨凇',
      snow: '积雪',
      wet_snow: '湿雪',
      mixed: '混合',
    };
    return texts[type] || type;
  };

  return (
    <div class="space-y-6">
      <div class="bg-white rounded-xl shadow-sm border border-slate-200 p-6">
        <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
          <div>
            <h2 class="text-2xl font-bold text-slate-800">覆冰风险分析</h2>
            <p class="text-slate-500 mt-1">监测塔架覆冰情况，评估风险等级</p>
          </div>
          <div class="flex items-center gap-4">
            <select
              value={state.selectedTowerId}
              onChange$={(e) => handleTowerChange((e.target as HTMLSelectElement).value)}
              class="px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
            >
              {state.towers.map((tower) => (
                <option key={tower.id} value={tower.id}>
                  {tower.name}
                </option>
              ))}
            </select>
            <button
              onClick$={handleAnalyze}
              disabled={state.isLoading}
              class="inline-flex items-center px-4 py-2 bg-cyan-600 text-white rounded-lg hover:bg-cyan-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 mr-2" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M4 2a1 1 0 011 1v2.101a7.002 7.002 0 0111.601 2.566 1 1 0 11-1.885.666A5.002 5.002 0 005.999 7H9a1 1 0 010 2H4a1 1 0 01-1-1V3a1 1 0 011-1zm.008 9.057a1 1 0 011.276.61A5.002 5.002 0 0014.001 13H11a1 1 0 110-2h5a1 1 0 011 1v5a1 1 0 11-2 0v-2.101a7.002 7.002 0 01-11.601-2.566 1 1 0 01.61-1.276z" clip-rule="evenodd" />
              </svg>
              立即分析
            </button>
          </div>
        </div>
      </div>

      {state.isLoading ? (
        <div class="flex items-center justify-center h-64">
          <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
        </div>
      ) : state.latestAnalysis ? (
        <>
          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
            <div class="bg-white rounded-xl shadow-sm p-6 border border-slate-200">
              <p class="text-sm font-medium text-slate-500">风险等级</p>
              <div class="mt-2 flex items-center">
                <span
                  class={`inline-flex items-center px-3 py-1 rounded-full text-sm font-semibold ${getRiskLevelColor(
                    state.latestAnalysis.riskLevel
                  )}`}
                >
                  {getRiskLevelText(state.latestAnalysis.riskLevel)}
                </span>
              </div>
              <p class="mt-2 text-2xl font-bold text-slate-800">
                {state.latestAnalysis.riskScore.toFixed(1)} 分
              </p>
            </div>

            <div class="bg-white rounded-xl shadow-sm p-6 border border-slate-200">
              <p class="text-sm font-medium text-slate-500">最大覆冰厚度</p>
              <p class="mt-2 text-3xl font-bold text-cyan-600">
                {state.latestAnalysis.maxIceThicknessMm.toFixed(1)} mm
              </p>
              <p class="mt-1 text-sm text-slate-500">
                平均: {state.latestAnalysis.avgIceThicknessMm.toFixed(1)} mm
              </p>
            </div>

            <div class="bg-white rounded-xl shadow-sm p-6 border border-slate-200">
              <p class="text-sm font-medium text-slate-500">覆冰类型</p>
              <p class="mt-2 text-2xl font-bold text-slate-800">
                {getIceTypeText(state.latestAnalysis.iceType)}
              </p>
              <p class="mt-1 text-sm text-slate-500">
                增长率: {state.latestAnalysis.accumulationRateMmh.toFixed(2)} mm/h
              </p>
            </div>

            <div class="bg-white rounded-xl shadow-sm p-6 border border-slate-200">
              <p class="text-sm font-medium text-slate-500">分析置信度</p>
              <p class="mt-2 text-3xl font-bold text-blue-600">
                {(state.latestAnalysis.confidence * 100).toFixed(0)}%
              </p>
              <p class="mt-1 text-sm text-slate-500">
                {state.latestAnalysis.rawDataPoints} 个数据点
              </p>
            </div>
          </div>

          <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <div class="bg-white rounded-xl shadow-sm border border-slate-200 overflow-hidden">
              <div class="px-6 py-4 border-b border-slate-200">
                <h3 class="text-lg font-semibold text-slate-800">风险预测</h3>
              </div>
              <div class="p-6">
                {state.latestAnalysis.predictions.length > 0 ? (
                  <div class="space-y-4">
                    {state.latestAnalysis.predictions.map((pred, index) => (
                      <div key={index} class="flex items-center justify-between p-3 bg-slate-50 rounded-lg">
                        <div>
                          <p class="text-sm text-slate-500">
                            {new Date(pred.predictionTime).toLocaleString('zh-CN')}
                          </p>
                          <p class="text-sm font-medium text-slate-800">
                            预测厚度: {pred.predictedThicknessMm.toFixed(1)} mm
                          </p>
                        </div>
                        <span
                          class={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getRiskLevelColor(
                            pred.riskLevel
                          )}`}
                        >
                          {getRiskLevelText(pred.riskLevel)}
                        </span>
                      </div>
                    ))}
                  </div>
                ) : (
                  <div class="text-center text-slate-400 py-8">
                    暂无预测数据
                  </div>
                )}
              </div>
            </div>

            <div class="bg-white rounded-xl shadow-sm border border-slate-200 overflow-hidden">
              <div class="px-6 py-4 border-b border-slate-200">
                <h3 class="text-lg font-semibold text-slate-800">缓解策略</h3>
              </div>
              <div class="p-6">
                <div class="space-y-4">
                  <div>
                    <h4 class="font-medium text-slate-800 mb-2">🚨 立即行动</h4>
                    <ul class="list-disc list-inside space-y-1 text-sm text-slate-600">
                      {state.latestAnalysis.mitigationStrategy.immediateActions.map((action, i) => (
                        <li key={i}>{action}</li>
                      ))}
                    </ul>
                  </div>
                  <div>
                    <h4 class="font-medium text-slate-800 mb-2">⏰ 短期措施</h4>
                    <ul class="list-disc list-inside space-y-1 text-sm text-slate-600">
                      {state.latestAnalysis.mitigationStrategy.shortTermActions.map((action, i) => (
                        <li key={i}>{action}</li>
                      ))}
                    </ul>
                  </div>
                  <div class="p-4 bg-blue-50 rounded-lg">
                    <div class="flex justify-between items-center">
                      <span class="text-sm font-medium text-blue-800">建议运行速度</span>
                      <span class="text-lg font-bold text-blue-600">
                        {state.latestAnalysis.mitigationStrategy.recommendedSpeedMs.toFixed(1)} m/s
                      </span>
                    </div>
                    <div class="flex justify-between items-center mt-2">
                      <span class="text-sm font-medium text-blue-800">是否建议停运</span>
                      <span
                        class={`px-2 py-1 rounded text-xs font-medium ${
                          state.latestAnalysis.mitigationStrategy.shutdownRecommended
                            ? 'bg-red-100 text-red-700'
                            : 'bg-green-100 text-green-700'
                        }`}
                      >
                        {state.latestAnalysis.mitigationStrategy.shutdownRecommended ? '是' : '否'}
                      </span>
                    </div>
                    <div class="flex justify-between items-center mt-2">
                      <span class="text-sm font-medium text-blue-800">预计除冰时间</span>
                      <span class="text-sm text-blue-600">
                        {state.latestAnalysis.mitigationStrategy.estimatedDeicingTimeHours.toFixed(1)} 小时
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <div class="bg-white rounded-xl shadow-sm border border-slate-200 overflow-hidden">
            <div class="px-6 py-4 border-b border-slate-200">
              <h3 class="text-lg font-semibold text-slate-800">历史分析记录</h3>
            </div>
            <div class="divide-y divide-slate-100">
              {state.analysisResults.slice(0, 10).map((result, index) => (
                <div key={index} class="px-6 py-4 flex items-center justify-between hover:bg-slate-50">
                  <div>
                    <p class="font-medium text-slate-800">
                      {new Date(result.analysisTime).toLocaleString('zh-CN')}
                    </p>
                    <p class="text-sm text-slate-500">
                      最大厚度: {result.maxIceThicknessMm.toFixed(1)}mm | 
                      {result.contributingFactors.slice(0, 3).join(', ')}
                    </p>
                  </div>
                  <span
                    class={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getRiskLevelColor(
                      result.riskLevel
                    )}`}
                  >
                    {getRiskLevelText(result.riskLevel)} ({result.riskScore.toFixed(1)})
                  </span>
                </div>
              ))}
            </div>
          </div>
        </>
      ) : (
        <div class="bg-white rounded-xl shadow-sm border border-slate-200 p-12 text-center text-slate-500">
          暂无覆冰分析数据
        </div>
      )}
    </div>
  );
});
