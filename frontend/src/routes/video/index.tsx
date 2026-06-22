import { component$, useStore, useTask$, $ } from '@builder.io/qwik';
import { Link } from '@builder.io/qwik-city';
import type { Camera, VideoVerificationRequest, VideoVerificationResult, Tower } from '~/types';
import apiService from '~/services/api';

export default component$(() => {
  const state = useStore({
    towers: [] as Tower[],
    cameras: [] as Camera[],
    requests: [] as VideoVerificationRequest[],
    results: [] as VideoVerificationResult[],
    selectedTowerId: '',
    activeTab: 'cameras',
    isLoading: true,
    error: '',
  });

  const loadData = $(async () => {
    state.isLoading = true;
    try {
      const [towersRes, camerasRes, requestsRes, resultsRes] = await Promise.all([
        apiService.getTowers(1, 10),
        apiService.getCameras(state.selectedTowerId || undefined),
        apiService.getVideoRequests(undefined, undefined, 1, 10),
        apiService.getVideoResults(undefined, undefined, 1, 10),
      ]);
      state.towers = towersRes.data;
      state.cameras = camerasRes;
      state.requests = requestsRes.data;
      state.results = resultsRes.data;
    } catch (error) {
      state.error = error instanceof Error ? error.message : '加载数据失败';
    } finally {
      state.isLoading = false;
    }
  });

  useTask$(() => {
    loadData();
  });

  const handleStartLiveStream = $(async (cameraId: string) => {
    try {
      await apiService.startLiveStream(cameraId);
      alert('直播已启动');
    } catch (error) {
      console.error('启动直播失败:', error);
    }
  });

  const handleRequestVerification = $(async (towerId: string) => {
    try {
      await apiService.requestAutoVerification(towerId);
      loadData();
      alert('复核请求已发送');
    } catch (error) {
      console.error('请求复核失败:', error);
    }
  });

  const getCameraTypeText = (type: string) => {
    const texts: Record<string, string> = {
      ptz: '云台球机',
      fixed: '固定摄像机',
      thermal: '热成像',
      dome: '半球摄像机',
      bullet: '枪式摄像机',
      fisheye: '鱼眼摄像机',
    };
    return texts[type] || type;
  };

  const getCameraStatusColor = (status: string) => {
    const colors: Record<string, string> = {
      online: 'bg-green-100 text-green-800',
      offline: 'bg-red-100 text-red-800',
      recording: 'bg-blue-100 text-blue-800',
      maintenance: 'bg-yellow-100 text-yellow-800',
      faulty: 'bg-orange-100 text-orange-800',
      disabled: 'bg-slate-100 text-slate-600',
    };
    return colors[status] || 'bg-slate-100 text-slate-800';
  };

  const getCameraStatusText = (status: string) => {
    const texts: Record<string, string> = {
      online: '在线',
      offline: '离线',
      recording: '录制中',
      maintenance: '维护中',
      faulty: '故障',
      disabled: '已禁用',
    };
    return texts[status] || status;
  };

  const getVerificationTypeText = (type: string) => {
    const texts: Record<string, string> = {
      ice_presence: '覆冰存在',
      ice_thickness: '覆冰厚度',
      structural_damage: '结构损伤',
      cable_condition: '线缆状况',
      wind_effect: '风致影响',
      general_inspection: '常规检查',
      alert_confirmation: '告警确认',
      incident_review: '事件复查',
    };
    return texts[type] || type;
  };

  const getVerificationStatusColor = (status: string) => {
    const colors: Record<string, string> = {
      pending: 'bg-yellow-100 text-yellow-800',
      in_progress: 'bg-blue-100 text-blue-800',
      completed: 'bg-green-100 text-green-800',
      expired: 'bg-slate-100 text-slate-600',
      cancelled: 'bg-slate-100 text-slate-500',
    };
    return colors[status] || 'bg-slate-100 text-slate-800';
  };

  const getVerificationStatusText = (status: string) => {
    const texts: Record<string, string> = {
      pending: '待处理',
      in_progress: '进行中',
      completed: '已完成',
      expired: '已过期',
      cancelled: '已取消',
    };
    return texts[status] || status;
  };

  const getPriorityColor = (priority: string) => {
    const colors: Record<string, string> = {
      low: 'bg-green-100 text-green-800',
      medium: 'bg-yellow-100 text-yellow-800',
      high: 'bg-orange-100 text-orange-800',
      urgent: 'bg-red-100 text-red-800',
      immediate: 'bg-red-200 text-red-900',
    };
    return colors[priority] || 'bg-slate-100 text-slate-800';
  };

  const getPriorityText = (priority: string) => {
    const texts: Record<string, string> = {
      low: '低',
      medium: '中',
      high: '高',
      urgent: '紧急',
      immediate: '立即',
    };
    return texts[priority] || priority;
  };

  const getTowerName = (towerId: string) => {
    const tower = state.towers.find((t) => t.id === towerId);
    return tower?.name || towerId;
  };

  const getCameraName = (cameraId: string) => {
    const camera = state.cameras.find((c) => c.id === cameraId);
    return camera?.name || cameraId;
  };

  return (
    <div class="space-y-6">
      <div class="bg-white rounded-xl shadow-sm border border-slate-200 p-6">
        <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
          <div>
            <h2 class="text-2xl font-bold text-slate-800">视频复核中心</h2>
            <p class="text-slate-500 mt-1">查看摄像头视频，进行AI和人工复核</p>
          </div>
          <div class="flex items-center gap-4">
            <select
              value={state.selectedTowerId}
              onChange$={(e) => {
                state.selectedTowerId = (e.target as HTMLSelectElement).value;
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
            {state.towers.length > 0 && (
              <button
                onClick$={() => handleRequestVerification(state.selectedTowerId || state.towers[0].id)}
                class="inline-flex items-center px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors"
              >
                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 mr-2" viewBox="0 0 20 20" fill="currentColor">
                  <path d="M2 6a2 2 0 012-2h6a2 2 0 012 2v8a2 2 0 01-2 2H4a2 2 0 01-2-2V6zm12.553 1.106A1 1 0 0014 8v4a1 1 0 00.553.894l2 1A1 1 0 0018 13V7a1 1 0 00-1.447-.894l-2 1z" />
                </svg>
                申请自动复核
              </button>
            )}
          </div>
        </div>

        <div class="mt-6 border-b border-slate-200">
          <nav class="flex space-x-8">
            {[
              { id: 'cameras', label: '摄像头列表' },
              { id: 'requests', label: '复核请求' },
              { id: 'results', label: '复核结果' },
            ].map((tab) => (
              <button
                key={tab.id}
                onClick$={() => (state.activeTab = tab.id)}
                class={`py-4 px-1 border-b-2 font-medium text-sm transition-colors ${
                  state.activeTab === tab.id
                    ? 'border-blue-500 text-blue-600'
                    : 'border-transparent text-slate-500 hover:text-slate-700 hover:border-slate-300'
                }`}
              >
                {tab.label}
                {tab.id === 'requests' && state.requests.filter((r) => r.status === 'pending').length > 0 && (
                  <span class="ml-2 px-2 py-0.5 bg-red-100 text-red-700 text-xs rounded-full">
                    {state.requests.filter((r) => r.status === 'pending').length}
                  </span>
                )}
              </button>
            ))}
          </nav>
        </div>
      </div>

      {state.isLoading ? (
        <div class="flex items-center justify-center h-64">
          <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
        </div>
      ) : state.activeTab === 'cameras' ? (
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {state.cameras.map((camera) => (
            <div key={camera.id} class="bg-white rounded-xl shadow-sm border border-slate-200 overflow-hidden">
              <div class="aspect-video bg-slate-900 relative">
                <div class="absolute inset-0 flex items-center justify-center text-slate-500">
                  <div class="text-center">
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-16 w-16 mx-auto mb-2 opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
                    </svg>
                    <p class="text-sm">视频预览区域</p>
                  </div>
                </div>
                <div class="absolute top-2 left-2">
                  <span
                    class={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${getCameraStatusColor(
                      camera.status
                    )}`}
                  >
                    {getCameraStatusText(camera.status)}
                  </span>
                </div>
                {camera.hasAiAnalysis && (
                  <div class="absolute top-2 right-2">
                    <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-purple-100 text-purple-700">
                      AI分析
                    </span>
                  </div>
                )}
              </div>
              <div class="p-4">
                <div class="flex items-center justify-between mb-2">
                  <h3 class="font-semibold text-slate-800">{camera.name}</h3>
                  <span class="text-xs text-slate-500">{getCameraTypeText(camera.cameraType)}</span>
                </div>
                <p class="text-sm text-slate-500 mb-2">{camera.location}</p>
                <p class="text-xs text-slate-400 mb-4">
                  安装位置: {camera.mountPosition} | {camera.resolution} | {camera.fps}fps
                </p>
                <div class="flex gap-2">
                  <button
                    onClick$={() => handleStartLiveStream(camera.id)}
                    disabled={camera.status !== 'online'}
                    class="flex-1 px-3 py-2 text-sm bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    查看直播
                  </button>
                  <button
                    onClick$={() => handleRequestVerification(camera.towerId)}
                    class="flex-1 px-3 py-2 text-sm bg-purple-100 text-purple-700 rounded-md hover:bg-purple-200 transition-colors"
                  >
                    申请复核
                  </button>
                </div>
              </div>
            </div>
          ))}
          {state.cameras.length === 0 && (
            <div class="col-span-full bg-white rounded-xl shadow-sm border border-slate-200 p-12 text-center text-slate-500">
              暂无摄像头数据
            </div>
          )}
        </div>
      ) : state.activeTab === 'requests' ? (
        <div class="bg-white rounded-xl shadow-sm border border-slate-200 overflow-hidden">
          <div class="divide-y divide-slate-100">
            {state.requests.map((request) => (
              <div key={request.id} class="p-6 hover:bg-slate-50 transition-colors">
                <div class="flex items-start justify-between">
                  <div class="flex-1">
                    <div class="flex items-center gap-3 mb-2">
                      <h3 class="font-semibold text-slate-800">
                        {getVerificationTypeText(request.requestType)} - {getTowerName(request.towerId)}
                      </h3>
                      <span
                        class={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getVerificationStatusColor(
                          request.status
                        )}`}
                      >
                        {getVerificationStatusText(request.status)}
                      </span>
                      <span
                        class={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getPriorityColor(
                          request.priority
                        )}`}
                      >
                        {getPriorityText(request.priority)}
                      </span>
                    </div>
                    <p class="text-sm text-slate-600 mb-2">{request.description}</p>
                    <div class="flex items-center gap-4 text-xs text-slate-500">
                      <span>摄像头: {getCameraName(request.cameraId)}</span>
                      <span>申请时间: {new Date(request.requestedAt).toLocaleString('zh-CN')}</span>
                      {request.expiresAt && (
                        <span>过期时间: {new Date(request.expiresAt).toLocaleString('zh-CN')}</span>
                      )}
                    </div>
                    {request.itemsToVerify.length > 0 && (
                      <div class="mt-3 flex flex-wrap gap-2">
                        {request.itemsToVerify.map((item, i) => (
                          <span key={i} class="px-2 py-0.5 bg-slate-100 text-slate-600 text-xs rounded">
                            {item}
                          </span>
                        ))}
                      </div>
                    )}
                  </div>
                  <div class="flex items-center gap-2 ml-6">
                    {request.status === 'pending' && (
                      <button class="px-3 py-1.5 text-sm bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors">
                        开始复核
                      </button>
                    )}
                    <Link
                      href={`/video/requests/${request.id}`}
                      class="px-3 py-1.5 text-sm text-blue-600 hover:bg-blue-50 rounded-md transition-colors"
                    >
                      详情
                    </Link>
                  </div>
                </div>
              </div>
            ))}
            {state.requests.length === 0 && (
              <div class="px-6 py-12 text-center text-slate-500">
                暂无复核请求
              </div>
            )}
          </div>
        </div>
      ) : (
        <div class="bg-white rounded-xl shadow-sm border border-slate-200 overflow-hidden">
          <div class="divide-y divide-slate-100">
            {state.results.map((result) => (
              <div key={result.id} class="p-6 hover:bg-slate-50 transition-colors">
                <div class="flex items-start justify-between">
                  <div class="flex-1">
                    <div class="flex items-center gap-3 mb-2">
                      <h3 class="font-semibold text-slate-800">
                        复核结果 - {getTowerName(result.towerId)}
                      </h3>
                      <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">
                        已完成
                      </span>
                      {result.aiConfidence !== undefined && (
                        <span class="text-sm text-slate-500">
                          AI置信度: {(result.aiConfidence * 100).toFixed(0)}%
                        </span>
                      )}
                    </div>
                    <div class="flex items-center gap-4 text-xs text-slate-500 mb-2">
                      <span>摄像头: {getCameraName(result.cameraId)}</span>
                      <span>
                        完成时间: {result.completedAt ? new Date(result.completedAt).toLocaleString('zh-CN') : '-'}
                      </span>
                      <span>
                        复核方式: {result.verificationMethod === 'ai_only' ? '仅AI' : 
                                  result.verificationMethod === 'human_only' ? '仅人工' :
                                  result.verificationMethod === 'ai_with_human_review' ? 'AI+人工' :
                                  result.verificationMethod === 'live_stream' ? '直播' :
                                  result.verificationMethod === 'snapshot' ? '快照' : '录像'}
                      </span>
                    </div>
                    {result.humanReviewRequired && (
                      <div class="mb-2">
                        <span class="px-2 py-0.5 bg-yellow-100 text-yellow-700 text-xs rounded">
                          需要人工复核
                          {result.humanReviewed && ` (已复核: ${result.reviewedBy || '未知'})`}
                        </span>
                      </div>
                    )}
                    {result.overallFindings.length > 0 && (
                      <div class="mt-3">
                        <p class="text-sm font-medium text-slate-700 mb-2">发现结果:</p>
                        <ul class="list-disc list-inside space-y-1">
                          {result.overallFindings.map((finding, i) => (
                            <li key={i} class="text-sm text-slate-600">{finding}</li>
                          ))}
                        </ul>
                      </div>
                    )}
                    {result.reviewNotes && (
                      <p class="mt-2 text-sm text-slate-600 bg-slate-50 p-2 rounded">
                        复核备注: {result.reviewNotes}
                      </p>
                    )}
                  </div>
                  <div class="ml-6">
                    <Link
                      href={`/video/results/${result.id}`}
                      class="px-3 py-1.5 text-sm text-blue-600 hover:bg-blue-50 rounded-md transition-colors"
                    >
                      详情
                    </Link>
                  </div>
                </div>
              </div>
            ))}
            {state.results.length === 0 && (
              <div class="px-6 py-12 text-center text-slate-500">
                暂无复核结果
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
});
