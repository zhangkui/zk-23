import { component$, useContext, useStore, $ } from '@builder.io/qwik';
import { useNavigate } from '@builder.io/qwik-city';
import { AUTH_CONTEXT } from '~/stores/auth';
import apiService from '~/services/api';
import websocketService from '~/services/websocket';

export default component$(() => {
  const auth = useContext(AUTH_CONTEXT);
  const nav = useNavigate();

  const form = useStore({
    username: '',
    password: '',
    remember: false,
  });

  const state = useStore({
    isSubmitting: false,
    error: '',
  });

  const handleSubmit = $(async () => {
    if (!form.username || !form.password) {
      state.error = '请输入用户名和密码';
      return;
    }

    state.isSubmitting = true;
    state.error = '';

    try {
      const response = await apiService.login({
        username: form.username,
        password: form.password,
      });

      auth.user = response.user;
      auth.token = response.token;
      auth.isAuthenticated = true;

      websocketService.connect(response.token).catch((err) => {
        console.error('WebSocket 连接失败:', err);
      });

      nav('/');
    } catch (error) {
      state.error = error instanceof Error ? error.message : '登录失败，请检查用户名和密码';
    } finally {
      state.isSubmitting = false;
    }
  });

  return (
    <div class="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 via-slate-50 to-slate-100">
      <div class="w-full max-w-md">
        <div class="text-center mb-8">
          <div class="text-5xl mb-4">🏔️</div>
          <h1 class="text-3xl font-bold text-slate-800 mb-2">索道塔架监测平台</h1>
          <p class="text-slate-600">振动结冰联动监测与停运决策系统</p>
        </div>

        <div class="bg-white rounded-2xl shadow-xl p-8">
          <h2 class="text-xl font-semibold text-slate-800 mb-6 text-center">用户登录</h2>

          {state.error && (
            <div class="mb-6 p-4 bg-red-50 border border-red-200 rounded-lg text-red-700 text-sm">
              {state.error}
            </div>
          )}

          <div class="space-y-5">
            <div>
              <label class="block text-sm font-medium text-slate-700 mb-2">用户名</label>
              <input
                type="text"
                value={form.username}
                onInput$={(e) => (form.username = (e.target as HTMLInputElement).value)}
                placeholder="请输入用户名"
                class="w-full px-4 py-3 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all"
                disabled={state.isSubmitting}
              />
            </div>

            <div>
              <label class="block text-sm font-medium text-slate-700 mb-2">密码</label>
              <input
                type="password"
                value={form.password}
                onInput$={(e) => (form.password = (e.target as HTMLInputElement).value)}
                placeholder="请输入密码"
                class="w-full px-4 py-3 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all"
                disabled={state.isSubmitting}
                onKeyPress$={(e) => {
                  if (e.key === 'Enter') {
                    handleSubmit();
                  }
                }}
              />
            </div>

            <div class="flex items-center justify-between">
              <label class="flex items-center">
                <input
                  type="checkbox"
                  checked={form.remember}
                  onChange$={(e) => (form.remember = (e.target as HTMLInputElement).checked)}
                  class="w-4 h-4 text-blue-600 border-slate-300 rounded focus:ring-blue-500"
                />
                <span class="ml-2 text-sm text-slate-600">记住我</span>
              </label>
              <a href="#" class="text-sm text-blue-600 hover:text-blue-700">忘记密码？</a>
            </div>

            <button
              onClick$={handleSubmit}
              disabled={state.isSubmitting}
              class="w-full py-3 px-4 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center"
            >
              {state.isSubmitting ? (
                <>
                  <div class="animate-spin rounded-full h-5 w-5 border-b-2 border-white mr-2"></div>
                  登录中...
                </>
              ) : (
                '登 录'
              )}
            </button>
          </div>

          <div class="mt-6 pt-6 border-t border-slate-200">
            <p class="text-xs text-slate-500 text-center">
              默认账号：admin / admin123
            </p>
          </div>
        </div>

        <div class="mt-8 text-center text-sm text-slate-500">
          <p>© 2024 山地索道安全监测系统 | 景区边缘服务器部署</p>
        </div>
      </div>
    </div>
  );
});
