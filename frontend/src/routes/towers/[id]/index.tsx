import { component$, useStore, useTask$, useSignal, $ } from '@builder.io/qwik';
import { routeLoader$ } from '@builder.io/qwik-city';
import { api } from '~/services/api';
import type { Tower, Sensor, VibrationData, WindSpeedData } from '~/types';
import StatusBadge from '~/components/StatusBadge';
import LineChart from '~/components/LineChart';
import GaugeChart from '~/components/GaugeChart';
import dayjs from 'dayjs';

export const useTowerData = routeLoader$(async ({ params }) => {
  try {
    const [tower, sensors, vibrationData, windSpeedData] = await Promise.all([
      api.towers.getById(params.id),
      api.sensors.getByTowerId(params.id),
      api.data.getVibrationHistory(params.id, { hours: 24 }),
      api.data.getWindSpeedHistory(params.id, { hours: 24 }),
    ]);
    return { tower, sensors, vibrationData, windSpeedData };
  } catch (error) {
    return { tower: null, sensors: [], vibrationData: [], windSpeedData: [] };
  }
});

export default component$(() => {
  const towerData = useTowerData();
  const state = useStore({
    activeTab: 'overview',
    vibrationChart: [] as { time: string; value: number }[],
    windChart: [] as { time: string; value: number }[],
  });

  useTask$(() => {
    if (towerData.value.vibrationData) {
      state.vibrationChart = towerData.value.vibrationData.map((d: VibrationData) => ({
        time: dayjs(d.timestamp).format('HH:mm'),
        value: d.amplitude,
      }));
    }
    if (towerData.value.windSpeedData) {
      state.windChart = towerData.value.windSpeedData.map((d: WindSpeedData) => ({
        time: dayjs(d.timestamp).format('HH:mm'),
        value: d.speed,
      }));
    }
  });

  const tower = towerData.value.tower as Tower | null;
  const sensors = towerData.value.sensors as Sensor[];

  if (!tower) {
    return (
      <div class="card p-12 text-center">
        <div class="text-gray-400 text-6xl mb-4">🏔️</div>
        <h3 class="text-xl font-semibold text-gray-700 mb-2">塔架不存在</h3>
        <p class="text-gray-500">未找到对应的塔架信息</p>
      </div>
    );
  }

  const sensorTypeLabels: Record<string, string> = {
    vibration: '振动传感器',
    wind_speed: '风速传感器',
    ice_detection: '覆冰传感器',
    temperature: '温度传感器',
    humidity: '湿度传感器',
  };

  return (
    <div class="space-y-6">
      <div class="flex items-center justify-between">
        <div>
          <div class="flex items-center gap-3 mb-2">
            <h1 class="text-2xl font-bold text-gray-900">{tower.name}</h1>
            <StatusBadge status={tower.status as any} />
          </div>
          <p class="text-gray-500">
            编号: {tower.code} | 海拔: {tower.elevation}m | 坐标: {tower.latitude.toFixed(4)}, {tower.longitude.toFixed(4)}
          </p>
        </div>
        <button class="btn btn-secondary" onClick$={() => history.back()}>
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M15 19l-7-7 7-7" />
          </svg>
          返回列表
        </button>
      </div>

      <div class="flex border-b border-gray-200 gap-1">
        {['overview', 'sensors', 'history', 'analysis'].map((tab) => (
          <button
            key={tab}
            class={`tab ${state.activeTab === tab ? 'active' : ''}`}
            onClick$={() => { state.activeTab = tab; }}
          >
            {tab === 'overview' ? '概览' : tab === 'sensors' ? '传感器' : tab === 'history' ? '历史数据' : '风险分析'}
          </button>
        ))}
      </div>

      {state.activeTab === 'overview' && (
        <div class="space-y-6">
          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
            <div class="stat-card">
              <GaugeChart
                value={tower.risk_score || 0}
                max={100}
                title="风险评分"
                unit="分"
                thresholds={[
                  { value: 60, color: '#22c55e' },
                  { value: 85, color: '#f59e0b' },
                  { value: 100, color: '#ef4444' },
                ]}
              />
            </div>
            <div class="stat-card">
              <div class="stat-value">{tower.ice_thickness?.toFixed(1) || '0.0'}</div>
              <div class="stat-label">覆冰厚度</div>
              <div class="stat-change up">mm</div>
            </div>
            <div class="stat-card">
              <div class="stat-value">{tower.current_wind_speed?.toFixed(1) || '0.0'}</div>
              <div class="stat-label">当前风速</div>
              <div class="stat-change up">m/s</div>
            </div>
            <div class="stat-card">
              <div class="stat-value">{tower.current_vibration?.toFixed(3) || '0.000'}</div>
              <div class="stat-label">振动振幅</div>
              <div class="stat-change up">mm</div>
            </div>
          </div>

          <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <div class="card">
              <div class="card-header">振动趋势 (24小时)</div>
              <div class="card-body">
                <LineChart
                  data={state.vibrationChart}
                  yAxisName="振幅 (mm)"
                  color="#3b82f6"
                  height="280px"
                />
              </div>
            </div>
            <div class="card">
              <div class="card-header">风速趋势 (24小时)</div>
              <div class="card-body">
                <LineChart
                  data={state.windChart}
                  yAxisName="风速 (m/s)"
                  color="#f59e0b"
                  height="280px"
                />
              </div>
            </div>
          </div>

          <div class="card">
            <div class="card-header">塔架信息</div>
            <div class="card-body">
              <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <div>
                  <label class="label">塔架型号</label>
                  <p class="text-gray-900 font-medium">{tower.model || '未知'}</p>
                </div>
                <div>
                  <label class="label">安装日期</label>
                  <p class="text-gray-900 font-medium">{tower.install_date ? dayjs(tower.install_date).format('YYYY-MM-DD') : '未知'}</p>
                </div>
                <div>
                  <label class="label">最后巡检</label>
                  <p class="text-gray-900 font-medium">{tower.last_inspection ? dayjs(tower.last_inspection).format('YYYY-MM-DD HH:mm') : '从未'}</p>
                </div>
                <div>
                  <label class="label">承载能力</label>
                  <p class="text-gray-900 font-medium">{tower.load_capacity || '未知'} kN</p>
                </div>
                <div>
                  <label class="label">设计风速</label>
                  <p class="text-gray-900 font-medium">{tower.design_wind_speed || '未知'} m/s</p>
                </div>
                <div>
                  <label class="label">设计冰厚</label>
                  <p class="text-gray-900 font-medium">{tower.design_ice_thickness || '未知'} mm</p>
                </div>
              </div>
              <div class="mt-6">
                <label class="label">备注</label>
                <p class="text-gray-700">{tower.description || '无备注信息'}</p>
              </div>
            </div>
          </div>
        </div>
      )}

      {state.activeTab === 'sensors' && (
        <div class="card">
          <div class="card-header flex items-center justify-between">
            <span>传感器列表</span>
            <span class="text-sm text-gray-500">共 {sensors.length} 个传感器</span>
          </div>
          <div class="overflow-x-auto">
            <table class="table">
              <thead>
                <tr>
                  <th>传感器编号</th>
                  <th>类型</th>
                  <th>安装位置</th>
                  <th>状态</th>
                  <th>最后读数</th>
                  <th>最后更新</th>
                </tr>
              </thead>
              <tbody>
                {sensors.map((sensor) => (
                  <tr key={sensor.id}>
                    <td class="font-mono text-sm">{sensor.code}</td>
                    <td>{sensorTypeLabels[sensor.type] || sensor.type}</td>
                    <td>{sensor.location || '未设置'}</td>
                    <td>
                      <StatusBadge status={sensor.status as any} />
                    </td>
                    <td>
                      {sensor.last_value !== null && sensor.last_value !== undefined
                        ? `${sensor.last_value} ${sensor.unit || ''}`
                        : '-'}
                    </td>
                    <td class="text-gray-500">
                      {sensor.last_reading ? dayjs(sensor.last_reading).format('YYYY-MM-DD HH:mm') : '-'}
                    </td>
                  </tr>
                ))}
                {sensors.length === 0 && (
                  <tr>
                    <td colSpan={6} class="text-center py-12 text-gray-400">
                      暂无传感器数据
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {state.activeTab === 'history' && (
        <div class="space-y-6">
          <div class="card">
            <div class="card-header">振动历史数据</div>
            <div class="card-body">
              <LineChart
                data={state.vibrationChart}
                yAxisName="振幅 (mm)"
                color="#3b82f6"
                height="350px"
              />
            </div>
          </div>
          <div class="card">
            <div class="card-header">风速历史数据</div>
            <div class="card-body">
              <LineChart
                data={state.windChart}
                yAxisName="风速 (m/s)"
                color="#f59e0b"
                height="350px"
              />
            </div>
          </div>
        </div>
      )}

      {state.activeTab === 'analysis' && (
        <div class="space-y-6">
          <div class="card">
            <div class="card-header">风险因素分析</div>
            <div class="card-body">
              <div class="space-y-4">
                <div>
                  <div class="flex justify-between mb-1">
                    <span class="text-sm font-medium text-gray-700">覆冰风险</span>
                    <span class="text-sm font-medium text-orange-600">{tower.ice_risk_level || '低'}</span>
                  </div>
                  <div class="progress">
                    <div class="progress-bar bg-orange-500" style={{ width: `${(tower.ice_thickness || 0) / 30 * 100}%` }}></div>
                  </div>
                </div>
                <div>
                  <div class="flex justify-between mb-1">
                    <span class="text-sm font-medium text-gray-700">风速风险</span>
                    <span class="text-sm font-medium text-blue-600">{tower.wind_risk_level || '低'}</span>
                  </div>
                  <div class="progress">
                    <div class="progress-bar bg-blue-500" style={{ width: `${(tower.current_wind_speed || 0) / 30 * 100}%` }}></div>
                  </div>
                </div>
                <div>
                  <div class="flex justify-between mb-1">
                    <span class="text-sm font-medium text-gray-700">振动风险</span>
                    <span class="text-sm font-medium text-red-600">{tower.vibration_risk_level || '低'}</span>
                  </div>
                  <div class="progress">
                    <div class="progress-bar bg-red-500" style={{ width: `${(tower.current_vibration || 0) / 0.5 * 100}%` }}></div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <div class="card">
            <div class="card-header">维护建议</div>
            <div class="card-body">
              <div class="space-y-3">
                {tower.risk_score && tower.risk_score >= 80 ? (
                  <div class="alert alert-danger">
                    <strong>立即停运:</strong> 塔架风险评分过高，建议立即停运并进行全面检查。
                  </div>
                ) : tower.risk_score && tower.risk_score >= 60 ? (
                  <div class="alert alert-warning">
                    <strong>加强监测:</strong> 塔架存在一定风险，建议增加监测频率，必要时安排巡检。
                  </div>
                ) : (
                  <div class="alert alert-success">
                    <strong>运行正常:</strong> 塔架状态良好，按正常计划进行维护即可。
                  </div>
                )}
                <div class="p-4 bg-gray-50 rounded-lg">
                  <h4 class="font-medium text-gray-900 mb-2">下次巡检建议</h4>
                  <p class="text-gray-600 text-sm">
                    建议在 {dayjs().add(30, 'day').format('YYYY-MM-DD')} 前安排一次常规巡检，
                    重点检查塔架基础、螺栓紧固情况和传感器工作状态。
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
});
