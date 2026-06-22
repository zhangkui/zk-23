import { component$, useContext, Slot, useVisibleTask$ } from '@builder.io/qwik';
import { Link, useLocation } from '@builder.io/qwik-city';
import { AUTH_CONTEXT } from '~/stores/auth';
import websocketService from '~/services/websocket';
import apiService from '~/services/api';

export default component$(() => {
  const auth = useContext(AUTH_CONTEXT);
  const location = useLocation();

  useVisibleTask$(({ track }) => {
    track(() => auth.isAuthenticated);

    if (auth.isAuthenticated && auth.token) {
      websocketService.connect(auth.token).catch((err) => {
        console.error('WebSocket 连接失败:', err);
      });

      websocketService.subscribe('alerts');
    } else {
      websocketService.disconnect();
    }

    return () => {
      websocketService.disconnect();
    };
  });

  const isLoginPage = location.url.pathname === '/login';

  if (isLoginPage) {
    return <Slot />;
  }

  if (auth.isLoading) {
    return (
      <div class="min-h-screen flex items-center justify-center bg-slate-50">
        <div class="text-center">
          <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"></div>
          <p class="text-slate-600">加载中...</p>
        </div>
      </div>
    );
  }

  if (!auth.isAuthenticated) {
    return (
      <div class="min-h-screen flex items-center justify-center bg-slate-50">
        <div class="text-center">
          <div class="bg-white rounded-lg shadow-lg p-8 max-w-md mx-auto">
            <h2 class="text-2xl font-bold text-slate-800 mb-4">请先登录</h2>
            <p class="text-slate-600 mb-6">您需要登录才能访问此系统</p>
            <Link
              href="/login"
              class="inline-block px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
            >
              去登录
            </Link>
          </div>
        </div>
      </div>
    );
  }

  const navItems = [
    { path: '/', label: '仪表盘', icon: 'dashboard' },
    { path: '/towers', label: '塔架管理', icon: 'tower' },
    { path: '/alerts', label: '告警中心', icon: 'alert' },
    { path: '/ice-risk', label: '覆冰风险', icon: 'ice' },
    { path: '/shutdown', label: '停运策略', icon: 'power' },
    { path: '/inspections', label: '巡检记录', icon: 'clipboard' },
    { path: '/weather', label: '天气分析', icon: 'cloud' },
    { path: '/video', label: '视频复核', icon: 'camera' },
  ];

  const handleLogout = async () => {
    try {
      await apiService.logout();
      auth.user = null;
      auth.token = null;
      auth.isAuthenticated = false;
    } catch (error) {
      console.error('登出失败:', error);
    }
  };

  return (
    <div class="min-h-screen flex">
      <aside class="w-64 bg-slate-900 text-white flex flex-col">
        <div class="p-6 border-b border-slate-700">
          <h1 class="text-xl font-bold text-white">索道塔架监测平台</h1>
          <p class="text-sm text-slate-400 mt-1">振动结冰联动监测系统</p>
        </div>

        <nav class="flex-1 p-4 space-y-1">
          {navItems.map((item) => {
            const isActive = location.url.pathname === item.path;
            return (
              <Link
                key={item.path}
                href={item.path}
                class={`flex items-center px-4 py-3 rounded-lg transition-colors ${
                  isActive
                    ? 'bg-blue-600 text-white'
                    : 'text-slate-300 hover:bg-slate-800 hover:text-white'
                }`}
              >
                <span class="mr-3 text-lg">{getIcon(item.icon)}</span>
                {item.label}
              </Link>
            );
          })}
        </nav>

        <div class="p-4 border-t border-slate-700">
          <div class="flex items-center">
            <div class="w-10 h-10 rounded-full bg-blue-500 flex items-center justify-center text-white font-semibold">
              {auth.user?.fullName?.charAt(0) || 'U'}
            </div>
            <div class="ml-3 flex-1">
              <p class="text-sm font-medium text-white">{auth.user?.fullName}</p>
              <p class="text-xs text-slate-400">{getRoleLabel(auth.user?.role)}</p>
            </div>
            <button
              onClick$={handleLogout}
              class="text-slate-400 hover:text-white transition-colors"
              title="退出登录"
            >
              <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M3 3a1 1 0 00-1 1v12a1 1 0 102 0V4a1 1 0 00-1-1zm10.293 9.293a1 1 0 001.414 1.414l3-3a1 1 0 000-1.414l-3-3a1 1 0 10-1.414 1.414L14.586 9H7a1 1 0 100 2h7.586l-1.293 1.293z" clip-rule="evenodd" />
              </svg>
            </button>
          </div>
        </div>
      </aside>

      <main class="flex-1 flex flex-col overflow-hidden">
        <header class="bg-white border-b border-slate-200 px-8 py-4 flex items-center justify-between">
          <div>
            <h2 class="text-xl font-semibold text-slate-800">
              {navItems.find((item) => item.path === location.url.pathname)?.label || '仪表盘'}
            </h2>
          </div>
          <div class="flex items-center space-x-4">
            <div class="flex items-center text-sm">
              <span
                class={`inline-block w-2 h-2 rounded-full mr-2 ${
                  websocketService.isConnected() ? 'bg-green-500' : 'bg-red-500'
                }`}
              ></span>
              <span class="text-slate-600">
                {websocketService.isConnected() ? '实时连接' : '连接断开'}
              </span>
            </div>
          </div>
        </header>

        <div class="flex-1 overflow-auto p-8">
          <Slot />
        </div>
      </main>
    </div>
  );
});

function getIcon(name: string): string {
  const icons: Record<string, string> = {
    dashboard: '📊',
    tower: '🏗️',
    alert: '⚠️',
    ice: '❄️',
    power: '⚡',
    clipboard: '📋',
    cloud: '🌤️',
    camera: '📹',
  };
  return icons[name] || '📄';
}

function getRoleLabel(role?: string): string {
  const labels: Record<string, string> = {
    admin: '系统管理员',
    engineer: '工程师',
    technician: '技术人员',
    operator: '操作员',
    viewer: '查看员',
  };
  return labels[role || ''] || role || '';
}
