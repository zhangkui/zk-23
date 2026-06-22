import { component$, useContextProvider, useStore, useTask$ } from '@builder.io/qwik';
import { QwikCityProvider, RouterOutlet, ServiceWorkerRegister } from '@builder.io/qwik-city';
import { AUTH_CONTEXT, initialAuthState, initializeAuth } from '~/stores/auth';
import websocketService from '~/services/websocket';
import './global.css';

export default component$(() => {
  const authState = useStore({ ...initialAuthState });

  useContextProvider(AUTH_CONTEXT, authState);

  useTask$(async () => {
    authState.isLoading = true;
    try {
      const { user, token } = await initializeAuth();
      if (user && token) {
        authState.user = user;
        authState.token = token;
        authState.isAuthenticated = true;

        websocketService.connect(token).catch((err) => {
          console.error('WebSocket 连接失败:', err);
        });
      }
    } catch (error) {
      authState.error = error instanceof Error ? error.message : '初始化失败';
    } finally {
      authState.isLoading = false;
    }
  });

  return (
    <QwikCityProvider>
      <head>
        <meta charSet="utf-8" />
        <link rel="manifest" href="/manifest.json" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <meta name="theme-color" content="#1e3a5f" />
        <meta name="description" content="山地索道塔架振动结冰联动监测与停运决策平台" />
        <title>索道塔架监测平台</title>
        <ServiceWorkerRegister />
      </head>
      <body class="bg-slate-50 text-slate-800 antialiased">
        <RouterOutlet />
      </body>
    </QwikCityProvider>
  );
});
