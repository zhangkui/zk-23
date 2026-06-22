import type { WebSocketMessage, Alert, Tower, SensorDataMessage } from '~/types';

type MessageHandler = (message: WebSocketMessage) => void;
type AlertHandler = (alert: Alert) => void;
type TowerStatusHandler = (tower: Tower) => void;
type SensorDataHandler = (data: SensorDataMessage) => void;

class WebSocketService {
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 10;
  private reconnectDelay = 1000;
  private heartbeatInterval: number | null = null;
  private isManualClose = false;

  private messageHandlers: Map<string, Set<MessageHandler>> = new Map();
  private alertHandlers: Set<AlertHandler> = new Set();
  private towerStatusHandlers: Set<TowerStatusHandler> = new Set();
  private sensorDataHandlers: Set<SensorDataHandler> = new Set();

  private subscribedChannels: Set<string> = new Set();
  private subscribedTowers: Set<string> = new Set();

  connect(token?: string): Promise<void> {
    return new Promise((resolve, reject) => {
      if (this.ws && this.ws.readyState === WebSocket.OPEN) {
        resolve();
        return;
      }

      this.isManualClose = false;

      const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      const host = window.location.host;
      let url = `${protocol}//${host}/ws`;

      if (token) {
        url += `?token=${encodeURIComponent(token)}`;
      }

      try {
        this.ws = new WebSocket(url);
      } catch (error) {
        reject(error);
        return;
      }

      this.ws.onopen = () => {
        console.log('WebSocket 连接成功');
        this.reconnectAttempts = 0;
        this.startHeartbeat();
        this.resubscribeAll();
        resolve();
      };

      this.ws.onmessage = (event) => {
        try {
          const message: WebSocketMessage = JSON.parse(event.data);
          this.handleMessage(message);
        } catch (error) {
          console.error('WebSocket 消息解析失败:', error);
        }
      };

      this.ws.onerror = (error) => {
        console.error('WebSocket 错误:', error);
        reject(error);
      };

      this.ws.onclose = (event) => {
        console.log('WebSocket 连接关闭:', event.code, event.reason);
        this.stopHeartbeat();

        if (!this.isManualClose && this.reconnectAttempts < this.maxReconnectAttempts) {
          this.reconnectAttempts++;
          const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1);
          console.log(`尝试重连 (${this.reconnectAttempts}/${this.maxReconnectAttempts})，延迟 ${delay}ms`);
          setTimeout(() => this.connect(token), delay);
        }
      };
    });
  }

  disconnect(): void {
    this.isManualClose = true;
    this.stopHeartbeat();
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
    this.subscribedChannels.clear();
    this.subscribedTowers.clear();
  }

  private startHeartbeat(): void {
    this.heartbeatInterval = window.setInterval(() => {
      if (this.ws && this.ws.readyState === WebSocket.OPEN) {
        this.send({
          type: 'heartbeat',
          data: { timestamp: Date.now() },
        });
      }
    }, 30000);
  }

  private stopHeartbeat(): void {
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval);
      this.heartbeatInterval = null;
    }
  }

  private resubscribeAll(): void {
    this.subscribedChannels.forEach((channel) => {
      this.send({ type: 'subscribe', channel });
    });

    this.subscribedTowers.forEach((towerId) => {
      this.send({ type: 'subscribe', towerId });
    });
  }

  private handleMessage(message: WebSocketMessage): void {
    const handlers = this.messageHandlers.get(message.type);
    if (handlers) {
      handlers.forEach((handler) => handler(message));
    }

    switch (message.type) {
      case 'alert':
        this.alertHandlers.forEach((handler) => handler(message.data as Alert));
        break;
      case 'tower_status':
        this.towerStatusHandlers.forEach((handler) => handler(message.data as Tower));
        break;
      case 'sensor_data':
        this.sensorDataHandlers.forEach((handler) => handler(message.data as SensorDataMessage));
        break;
      case 'heartbeat':
        break;
      case 'error':
        console.error('WebSocket 错误消息:', message.data);
        break;
    }
  }

  private send(message: WebSocketMessage): void {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    } else {
      console.warn('WebSocket 未连接，无法发送消息');
    }
  }

  subscribe(channel: string): void {
    this.subscribedChannels.add(channel);
    this.send({ type: 'subscribe', channel });
  }

  unsubscribe(channel: string): void {
    this.subscribedChannels.delete(channel);
    this.send({ type: 'unsubscribe', channel });
  }

  subscribeTower(towerId: string): void {
    this.subscribedTowers.add(towerId);
    this.send({ type: 'subscribe', towerId });
  }

  unsubscribeTower(towerId: string): void {
    this.subscribedTowers.delete(towerId);
    this.send({ type: 'unsubscribe', towerId });
  }

  onMessage(type: string, handler: MessageHandler): () => void {
    if (!this.messageHandlers.has(type)) {
      this.messageHandlers.set(type, new Set());
    }
    this.messageHandlers.get(type)!.add(handler);

    return () => {
      this.messageHandlers.get(type)?.delete(handler);
    };
  }

  onAlert(handler: AlertHandler): () => void {
    this.alertHandlers.add(handler);
    return () => this.alertHandlers.delete(handler);
  }

  onTowerStatus(handler: TowerStatusHandler): () => void {
    this.towerStatusHandlers.add(handler);
    return () => this.towerStatusHandlers.delete(handler);
  }

  onSensorData(handler: SensorDataHandler): () => void {
    this.sensorDataHandlers.add(handler);
    return () => this.sensorDataHandlers.delete(handler);
  }

  isConnected(): boolean {
    return this.ws !== null && this.ws.readyState === WebSocket.OPEN;
  }

  getReadyState(): number {
    return this.ws?.readyState ?? WebSocket.CLOSED;
  }
}

export const websocketService = new WebSocketService();
export default websocketService;
