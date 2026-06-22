import { component$, useStore, useTask$, $ } from '@builder.io/qwik';
import type { WeatherImpactAnalysis, WeatherAlert, WeatherForecast, Tower } from '~/types';
import apiService from '~/services/api';

export default component$(() => {
  const state = useStore({
    towers: [] as Tower[],
    selectedTowerId: '',
    analysis: null as WeatherImpactAnalysis | null,
    alerts: [] as WeatherAlert[],
    forecast: null as WeatherForecast | null,
    isLoading: true,
    error: '',
    days: 7,
  });

  const loadData = $(async () => {
    state.isLoading = true;
    try {
      const towersRes = await apiService.getTowers(1, 10);
      state.towers = towersRes.data;

      if (towersRes.data.length > 0) {
        state.selectedTowerId = state.selectedTowerId || towersRes.data[0].id;
        const [analysisRes, alertsRes, forecastRes] = await Promise.all([
          apiService.getWeatherAnalysis(state.selectedTowerId, state.days),
          apiService.getWeatherAlerts(state.selectedTowerId, true, 1, 10),
          apiService.getWeatherForecast(state.selectedTowerId, 48),
        ]);
        state.analysis = analysisRes;
        state.alerts = alertsRes.data;
        state.forecast = forecastRes;
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

  const getImpactRatingColor = (rating: string) => {
    const colors: Record<string, string> = {
      negligible: 'bg-green-100 text-green-800',
      minor: 'bg-blue-100 text-blue-800',
      moderate: 'bg-yellow-100 text-yellow-800',
      major: 'bg-orange-100 text-orange-800',
      significant: 'bg-orange-200 text-orange-900',
      severe: 'bg-red-100 text-red-800',
      extreme: 'bg-red-200 text-red-900',
    };
    return colors[rating] || 'bg-slate-100 text-slate-800';
  };

  const getImpactRatingText = (rating: string) => {
    const texts: Record<string, string> = {
      negligible: '可忽略',
      minor: '轻微',
      moderate: '中等',
      major: '较大',
      significant: '显著',
      severe: '严重',
      extreme: '极端',
    };
    return texts[rating] || rating;
  };

  const getAlertTypeText = (type: string) => {
    const texts: Record<string, string> = {
      wind_warning: '大风预警',
      ice_warning: '结冰预警',
      ice_storm_warning: '冰暴预警',
      blizzard_warning: '暴雪预警',
      freezing_rain_warning: '冻雨预警',
      extreme_cold_warning: '极寒预警',
      thunderstorm_warning: '雷暴预警',
      heavy_snow_warning: '大雪预警',
      avalanche_warning: '雪崩预警',
      frost_warning: '霜冻预警',
      dense_fog_warning: '大雾预警',
      high_wind_warning: '强风预警',
    };
    return texts[type] || type;
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

  const getPrecipitationTypeText = (type: string) => {
    const texts: Record<string, string> = {
      none: '无',
      rain: '雨',
      snow: '雪',
      sleet: '雨夹雪',
      freezing_rain: '冻雨',
      hail: '冰雹',
      drizzle: '毛毛雨',
      mixed: '混合',
    };
    return texts[type] || type;
  };

  return (
    <div class="space-y-6">
      <div class="bg-white rounded-xl shadow-sm border border-slate-200 p-6">
        <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
          <div>
            <h2 class="text-2xl font-bold text-slate-800">天气影响分析</h2>
            <p class="text-slate-500 mt-1">分析天气对索道运行的影响，提供预警和建议</p>
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
            <select
              value={state.days}
              onChange$={(e) => {
                state.days = parseInt((e.target as HTMLSelectElement).value);
                loadData();
              }}
              class="px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
            >
              <option value="1">最近 1 天</option>
              <option value="3">最近 3 天</option>
              <option value="7">最近 7 天</option>
              <option value="14">最近 14 天</option>
              <option value="30">最近 30 天</option>
            </select>
          </div>
        </div>
      </div>

      {state.isLoading ? (
        <div class="flex items-center justify-center h-64">
          <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
        </div>
      ) : state.analysis ? (
        <>
          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
            <div class="bg-white rounded-xl shadow-sm p-6 border border-slate-200">
              <p class="text-sm font-medium text-slate-500">整体风险等级</p>
              <div class="mt-2">
                <span
                  class={`inline-flex items-center px-3 py-1 rounded-full text-sm font-semibold ${getRiskLevelColor(
                    state.analysis.overallRisk
                  )}`}
                >
                  {getRiskLevelText(state.analysis.overallRisk)}
                </span>
              </div>
              <p class="mt-2 text-sm text-slate-500">
                分析周期: {new Date(state.analysis.analysisPeriodStart).toLocaleDateString('zh-CN')} - {new Date(state.analysis.analysisPeriodEnd).toLocaleDateString('zh-CN')}
              </p>
            </div>

            <div class="bg-white rounded-xl shadow-sm p-6 border border-slate-200">
              <p class="text-sm font-medium text-slate-500">影响等级</p>
              <div class="mt-2">
                <span
                  class={`inline-flex items-center px-3 py-1 rounded-full text-sm font-semibold ${getImpactRatingColor(
                    state.analysis.impactRating
                  )}`}
                >
                  {getImpactRatingText(state.analysis.impactRating)}
                </span>
              </div>
              <p class="mt-2 text-sm text-slate-500">
                置信度: {(state.analysis.confidence * 100).toFixed(0)}%
              </p>
            </div>

            <div class="bg-white rounded-xl shadow-sm p-6 border border-slate-200">
              <p class="text-sm font-medium text-slate-500">平均温度</p>
              <p class="mt-2 text-3xl font-bold text-blue-600">
                {state.analysis.weatherSummary.avgTemperatureC.toFixed(1)}°C
              </p>
              <p class="mt-1 text-sm text-slate-500">
                最低 {state.analysis.weatherSummary.minTemperatureC.toFixed(1)}°C / 最高 {state.analysis.weatherSummary.maxTemperatureC.toFixed(1)}°C
              </p>
            </div>

            <div class="bg-white rounded-xl shadow-sm p-6 border border-slate-200">
              <p class="text-sm font-medium text-slate-500">平均风速</p>
              <p class="mt-2 text-3xl font-bold text-cyan-600">
                {state.analysis.weatherSummary.avgWindSpeedMs.toFixed(1)} m/s
              </p>
              <p class="mt-1 text-sm text-slate-500">
                最大 {state.analysis.weatherSummary.maxWindSpeedMs.toFixed(1)} m/s
              </p>
            </div>
          </div>

          <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
            <div class="lg:col-span-2 bg-white rounded-xl shadow-sm border border-slate-200 overflow-hidden">
              <div class="px-6 py-4 border-b border-slate-200">
                <h3 class="text-lg font-semibold text-slate-800">天气统计摘要</h3>
              </div>
              <div class="p-6">
                <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                  <div class="text-center p-4 bg-blue-50 rounded-lg">
                    <p class="text-2xl font-bold text-blue-600">{state.analysis.weatherSummary.daysWithIceRisk}</p>
                    <p class="text-sm text-slate-600">结冰风险天数</p>
                  </div>
                  <div class="text-center p-4 bg-cyan-50 rounded-lg">
                    <p class="text-2xl font-bold text-cyan-600">{state.analysis.weatherSummary.daysWithHighWind}</p>
                    <p class="text-sm text-slate-600">大风天数</p>
                  </div>
                  <div class="text-center p-4 bg-orange-50 rounded-lg">
                    <p class="text-2xl font-bold text-orange-600">{state.analysis.weatherSummary.daysWithExtremeTemp}</p>
                    <p class="text-sm text-slate-600">极端温度天数</p>
                  </div>
                  <div class="text-center p-4 bg-purple-50 rounded-lg">
                    <p class="text-2xl font-bold text-purple-600">{state.analysis.weatherSummary.daysWithPrecipitation}</p>
                    <p class="text-sm text-slate-600">降水天数</p>
                  </div>
                </div>
                <div class="mt-6">
                  <h4 class="font-medium text-slate-800 mb-3">风险类型分布</h4>
                  <div class="space-y-2">
                    {state.analysis.riskByType.map(([type, level, count], index) => (
                      <div key={index} class="flex items-center justify-between p-3 bg-slate-50 rounded-lg">
                        <span class="text-sm text-slate-600">
                          {type === 'ice' ? '结冰风险' : 
                           type === 'high_wind' ? '大风风险' :
                           type === 'extreme_temperature' ? '极端温度风险' :
                           type === 'heavy_precipitation' ? '强降水风险' : '一般风险'}
                        </span>
                        <div class="flex items-center gap-3">
                          <span
                            class={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${getRiskLevelColor(level)}`}
                          >
                            {getRiskLevelText(level)}
                          </span>
                          <span class="text-sm font-medium text-slate-800">{count} 次</span>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            </div>

            <div class="bg-white rounded-xl shadow-sm border border-slate-200 overflow-hidden">
              <div class="px-6 py-4 border-b border-slate-200">
                <h3 class="text-lg font-semibold text-slate-800">天气预警</h3>
              </div>
              <div class="p-6">
                {state.alerts.length > 0 ? (
                  <div class="space-y-3">
                    {state.alerts.slice(0, 5).map((alert) => (
                      <div key={alert.id} class="p-3 bg-slate-50 rounded-lg">
                        <div class="flex items-center justify-between mb-1">
                          <span class="font-medium text-slate-800 text-sm">
                            {getAlertTypeText(alert.alertType)}
                          </span>
                          <span
                            class={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${getSeverityColor(alert.severity)}`}
                          >
                            {getSeverityText(alert.severity)}
                          </span>
                        </div>
                        <p class="text-sm text-slate-600">{alert.headline}</p>
                        <p class="text-xs text-slate-400 mt-1">
                          {new Date(alert.effectiveStart).toLocaleString('zh-CN')} - {new Date(alert.effectiveEnd).toLocaleString('zh-CN')}
                        </p>
                      </div>
                    ))}
                  </div>
                ) : (
                  <div class="text-center text-slate-400 py-8">
                    暂无天气预警
                  </div>
                )}
              </div>
            </div>
          </div>

          {state.forecast && (
            <div class="bg-white rounded-xl shadow-sm border border-slate-200 overflow-hidden">
              <div class="px-6 py-4 border-b border-slate-200">
                <h3 class="text-lg font-semibold text-slate-800">未来 48 小时预报</h3>
                <p class="text-sm text-slate-500 mt-1">
                  来源: {state.forecast.source} | 发布时间: {new Date(state.forecast.forecastTime).toLocaleString('zh-CN')}
                </p>
              </div>
              <div class="p-6">
                <div class="grid grid-cols-6 md:grid-cols-12 gap-2">
                  {state.forecast.hourlyForecast.slice(0, 12).map((hour, index) => (
                    <div key={index} class="text-center p-3 bg-slate-50 rounded-lg">
                      <p class="text-xs text-slate-500">
                        {new Date(hour.timestamp).getHours()}:00
                      </p>
                      <p class="text-lg font-bold text-blue-600 my-1">
                        {hour.temperatureC.toFixed(0)}°
                      </p>
                      <p class="text-xs text-cyan-600">
                        {hour.windSpeedMs.toFixed(1)}m/s
                      </p>
                      <p class="text-xs text-slate-500 mt-1">
                        {getPrecipitationTypeText(hour.precipitationType)}
                      </p>
                      {hour.precipitationProbabilityPercent > 0 && (
                        <p class="text-xs text-blue-500">
                          💧 {hour.precipitationProbabilityPercent}%
                        </p>
                      )}
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}

          {state.analysis.mitigationRecommendations.length > 0 && (
            <div class="bg-white rounded-xl shadow-sm border border-slate-200 overflow-hidden">
              <div class="px-6 py-4 border-b border-slate-200">
                <h3 class="text-lg font-semibold text-slate-800">缓解建议</h3>
              </div>
              <div class="p-6">
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                  {state.analysis.mitigationRecommendations.map((rec, index) => (
                    <div key={index} class="p-4 bg-blue-50 rounded-lg">
                      <div class="flex items-start justify-between mb-2">
                        <span class="text-sm font-medium text-blue-800">
                          {rec.riskType === 'ice' ? '结冰风险' : 
                           rec.riskType === 'high_wind' ? '大风风险' :
                           rec.riskType === 'extreme_temperature' ? '极端温度风险' :
                           rec.riskType === 'heavy_precipitation' ? '强降水风险' : '一般风险'}
                        </span>
                        <span
                          class={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${
                            rec.priority === 'critical' ? 'bg-red-100 text-red-800' :
                            rec.priority === 'high' ? 'bg-orange-100 text-orange-800' :
                            rec.priority === 'medium' ? 'bg-yellow-100 text-yellow-800' :
                            'bg-green-100 text-green-800'
                          }`}
                        >
                          {rec.priority === 'critical' ? '紧急' :
                           rec.priority === 'high' ? '高' :
                           rec.priority === 'medium' ? '中' : '低'}
                        </span>
                      </div>
                      <p class="text-sm text-blue-700">{rec.action}</p>
                      <div class="flex items-center gap-4 mt-2 text-xs text-blue-600">
                        <span>预计成本: ¥{rec.estimatedCost.toLocaleString()}</span>
                        <span>有效性: {(rec.effectiveness * 100).toFixed(0)}%</span>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}

          <div class="bg-white rounded-xl shadow-sm border border-slate-200 overflow-hidden">
            <div class="px-6 py-4 border-b border-slate-200">
              <h3 class="text-lg font-semibold text-slate-800">影响评估</h3>
            </div>
            <div class="p-6">
              <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div class="p-4 bg-slate-50 rounded-lg">
                  <h4 class="font-medium text-slate-800 mb-2">运营影响</h4>
                  <p class="text-sm text-slate-600">{state.analysis.impactAssessment.operationalImpact}</p>
                </div>
                <div class="p-4 bg-slate-50 rounded-lg">
                  <h4 class="font-medium text-slate-800 mb-2">结构影响</h4>
                  <p class="text-sm text-slate-600">{state.analysis.impactAssessment.structuralImpact}</p>
                </div>
                <div class="p-4 bg-slate-50 rounded-lg">
                  <h4 class="font-medium text-slate-800 mb-2">维护影响</h4>
                  <p class="text-sm text-slate-600">{state.analysis.impactAssessment.maintenanceImpact}</p>
                </div>
              </div>
              <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mt-4">
                <div class="text-center p-4 bg-orange-50 rounded-lg">
                  <p class="text-2xl font-bold text-orange-600">
                    +{state.analysis.impactAssessment.estimatedCostIncreasePercent}%
                  </p>
                  <p class="text-sm text-slate-600">成本增加</p>
                </div>
                <div class="text-center p-4 bg-red-50 rounded-lg">
                  <p class="text-2xl font-bold text-red-600">
                    {state.analysis.impactAssessment.estimatedDowntimeHours}h
                  </p>
                  <p class="text-sm text-slate-600">预计停运</p>
                </div>
                <div class="col-span-2 p-4 bg-yellow-50 rounded-lg">
                  <h4 class="font-medium text-yellow-800 mb-1">乘客影响</h4>
                  <p class="text-sm text-yellow-700">{state.analysis.impactAssessment.passengerImpact}</p>
                </div>
              </div>
            </div>
          </div>
        </>
      ) : (
        <div class="bg-white rounded-xl shadow-sm border border-slate-200 p-12 text-center text-slate-500">
          暂无天气分析数据
        </div>
      )}
    </div>
  );
});
