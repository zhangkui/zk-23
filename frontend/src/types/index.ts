export type UUID = string;

export interface User {
  id: UUID;
  username: string;
  fullName: string;
  email: string;
  role: UserRole;
  department: string;
  phone: string;
  isActive: boolean;
  createdAt: string;
}

export type UserRole = 'admin' | 'engineer' | 'technician' | 'operator' | 'viewer';

export interface LoginRequest {
  username: string;
  password: string;
}

export interface LoginResponse {
  token: string;
  user: User;
}

export interface JwtClaims {
  sub: UUID;
  username: string;
  role: UserRole;
  exp: number;
  iat: number;
}

export interface Tower {
  id: UUID;
  name: string;
  code: string;
  location: {
    lat: number;
    lng: number;
    altitude: number;
  };
  status: TowerStatus;
  height: number;
  type: string;
  installationDate: string;
  lastInspectionDate: string;
  nextInspectionDate: string;
  description: string;
  sensors: Sensor[];
  cameras: Camera[];
  createdAt: string;
  updatedAt: string;
}

export type TowerStatus = 'normal' | 'warning' | 'danger' | 'maintenance' | 'offline';

export interface Sensor {
  id: UUID;
  towerId: UUID;
  type: SensorType;
  name: string;
  model: string;
  status: SensorStatus;
  installationLocation: string;
  samplingRate: number;
  lastCalibrationDate: string;
  createdAt: string;
}

export type SensorType = 'vibration' | 'wind_speed' | 'wind_direction' | 'temperature' | 'humidity' | 'ice_detection' | 'tilt' | 'strain';

export type SensorStatus = 'online' | 'offline' | 'maintenance' | 'faulty' | 'calibrating';

export interface Camera {
  id: UUID;
  towerId: UUID;
  deviceId: string;
  name: string;
  location: string;
  mountPosition: string;
  cameraType: CameraType;
  status: CameraStatus;
  rtspUrl?: string;
  httpUrl?: string;
  resolution: string;
  fps: number;
  hasAiAnalysis: boolean;
  aiModelVersion?: string;
  lastOnline?: string;
  createdAt: string;
}

export type CameraType = 'ptz' | 'fixed' | 'thermal' | 'dome' | 'bullet' | 'fisheye';

export type CameraStatus = 'online' | 'offline' | 'recording' | 'maintenance' | 'faulty' | 'disabled';

export interface VibrationReading {
  sensorId: UUID;
  towerId: UUID;
  timestamp: string;
  velocity_mm_s: number;
  acceleration_g: number;
  frequency_hz: number;
  direction: string;
  temperature: number;
  rawSpectrum?: number[];
}

export interface WindSpeedReading {
  sensorId: UUID;
  towerId: UUID;
  timestamp: string;
  speed_ms: number;
  gust_ms: number;
  direction_deg: number;
  temperature: number;
  quality: number;
}

export interface IceDetectionData {
  sensorId: UUID;
  towerId: UUID;
  timestamp: string;
  thickness_mm: number;
  type: IceType;
  load_kg_m2: number;
  temperature: number;
  humidity: number;
  quality: number;
}

export type IceType = 'none' | 'rime' | 'glaze' | 'snow' | 'wet_snow' | 'mixed';

export interface WeatherData {
  id: UUID;
  towerId: UUID;
  timestamp: string;
  temperature_c: number;
  humidity_percent: number;
  pressure_hpa: number;
  wind_speed_ms: number;
  wind_direction_deg: number;
  wind_gust_ms?: number;
  precipitation_mm?: number;
  precipitation_type: PrecipitationType;
  visibility_m?: number;
  cloud_cover_percent?: number;
}

export type PrecipitationType = 'none' | 'rain' | 'snow' | 'sleet' | 'freezing_rain' | 'hail' | 'drizzle' | 'mixed';

export interface Alert {
  id: UUID;
  towerId: UUID;
  type: AlertType;
  severity: AlertSeverity;
  status: AlertStatus;
  title: string;
  message: string;
  data?: Record<string, any>;
  triggeredAt: string;
  acknowledgedAt?: string;
  acknowledgedBy?: UUID;
  resolvedAt?: string;
  resolvedBy?: UUID;
  resolutionNotes?: string;
  source: AlertSource;
  relatedAlertIds?: UUID[];
}

export type AlertType = 'vibration' | 'wind_speed' | 'ice_detection' | 'temperature' | 'tilt' | 'strain' | 'system' | 'video' | 'weather';

export type AlertSeverity = 'low' | 'minor' | 'moderate' | 'severe' | 'extreme';

export type AlertStatus = 'active' | 'acknowledged' | 'resolved' | 'expired';

export type AlertSource = 'sensor' | 'ai_analysis' | 'manual' | 'system' | 'weather';

export interface IceAnalysisResult {
  id: UUID;
  towerId: UUID;
  analysisTime: string;
  avgIceThicknessMm: number;
  maxIceThicknessMm: number;
  riskScore: number;
  riskLevel: RiskLevel;
  iceType: IceType;
  accumulationRateMmh: number;
  contributingFactors: string[];
  predictions: IcePrediction[];
  mitigationStrategy: IceMitigationStrategy;
  confidence: number;
  rawDataPoints: number;
}

export type RiskLevel = 'none' | 'low' | 'medium' | 'high' | 'critical' | 'extreme';

export interface IcePrediction {
  predictionTime: string;
  predictedThicknessMm: number;
  riskLevel: RiskLevel;
  confidence: number;
}

export interface IceMitigationStrategy {
  immediateActions: string[];
  shortTermActions: string[];
  longTermActions: string[];
  recommendedSpeedMs: number;
  shutdownRecommended: boolean;
  estimatedDeicingTimeHours: number;
  safeIceThresholdMm: number;
}

export interface ShutdownStrategy {
  id: UUID;
  towerId: UUID;
  strategyType: ShutdownType;
  status: StrategyStatus;
  severity: StrategySeverity;
  title: string;
  description: string;
  triggerConditions: TriggerCondition[];
  actionSteps: ActionStep[];
  estimatedDurationMinutes: number;
  affectedArea: string;
  safetyMeasures: string[];
  createdAt: string;
  triggeredAt?: string;
  executedBy?: UUID;
  approvedBy?: UUID;
  autoApprove: boolean;
}

export type ShutdownType = 'preemptive' | 'emergency' | 'scheduled' | 'manual' | 'recovery';

export type StrategyStatus = 'draft' | 'pending_approval' | 'approved' | 'triggered' | 'executing' | 'completed' | 'cancelled' | 'expired';

export type StrategySeverity = 'advisory' | 'watch' | 'warning' | 'severe' | 'extreme';

export interface TriggerCondition {
  id: UUID;
  conditionType: ConditionType;
  metric: string;
  operator: ConditionOperator;
  threshold: number;
  unit: string;
  durationMinutes?: number;
  description: string;
}

export type ConditionType = 'vibration_velocity' | 'vibration_frequency' | 'wind_speed' | 'wind_gust' | 'ice_thickness' | 'ice_accumulation_rate' | 'temperature' | 'combined_risk' | 'video_confirmation';

export type ConditionOperator = 'greater_than' | 'less_than' | 'greater_than_or_equal' | 'less_than_or_equal' | 'equal';

export interface ActionStep {
  id: UUID;
  stepNumber: number;
  action: string;
  responsibleRole: string;
  estimatedDurationMinutes: number;
  completed: boolean;
  completedAt?: string;
  notes?: string;
}

export interface InspectionRecord {
  id: UUID;
  towerId: UUID;
  type: InspectionType;
  status: InspectionStatus;
  overallCondition: OverallCondition;
  inspectorId?: UUID;
  inspectionDate: string;
  startTime: string;
  endTime?: string;
  weatherConditions?: string;
  equipmentUsed?: string[];
  findings: InspectionFinding[];
  maintenanceTasks: MaintenanceTask[];
  recommendations: string[];
  notes?: string;
  attachments: string[];
  createdAt: string;
}

export type InspectionType = 'routine' | 'comprehensive' | 'post_incident' | 'post_storm' | 'emergency' | 'specialized';

export type InspectionStatus = 'scheduled' | 'in_progress' | 'completed' | 'cancelled' | 'overdue';

export type OverallCondition = 'excellent' | 'good' | 'fair' | 'poor' | 'critical' | 'safety';

export interface InspectionFinding {
  id: UUID;
  category: CheckCategory;
  location: string;
  description: string;
  severity: FindingSeverity;
  status: FindingStatus;
  priority: Priority;
  photos?: string[];
  detectedAt?: string;
  resolvedAt?: string;
  resolutionNotes?: string;
  relatedTaskId?: UUID;
}

export type CheckCategory = 'foundation' | 'structure' | 'cable' | 'sensor' | 'machinery' | 'electrical' | 'safety' | 'general';

export type FindingSeverity = 'low' | 'medium' | 'high' | 'critical' | 'safety';

export type FindingStatus = 'open' | 'in_progress' | 'resolved' | 'monitor' | 'closed';

export type Priority = 'low' | 'medium' | 'high' | 'urgent' | 'immediate';

export interface MaintenanceTask {
  id: UUID;
  taskType: MaintenanceType;
  title: string;
  description: string;
  priority: MaintenancePriority;
  status: TaskStatus;
  estimatedDurationHours: number;
  actualDurationHours?: number;
  dueDate: string;
  scheduledDate?: string;
  completedAt?: string;
  assignedTo?: UUID;
  completedBy?: UUID;
  partsRequired?: string[];
  toolsRequired?: string[];
  safetyPrecautions?: string[];
  notes?: string;
  createdAt: string;
}

export type MaintenanceType = 'repair' | 'replacement' | 'lubrication' | 'adjustment' | 'calibration' | 'cleaning' | 'inspection' | 'upgrade';

export type MaintenancePriority = 'low' | 'medium' | 'high' | 'critical';

export type TaskStatus = 'pending' | 'scheduled' | 'in_progress' | 'completed' | 'cancelled' | 'overdue';

export interface WeatherImpactAnalysis {
  id: UUID;
  towerId?: UUID;
  analysisTime: string;
  analysisPeriodStart: string;
  analysisPeriodEnd: string;
  dataPointsCount: number;
  weatherSummary: WeatherSummary;
  alerts: WeatherAlert[];
  overallRisk: RiskLevel;
  impactRating: ImpactRating;
  riskByType: [WeatherRiskType, RiskLevel, number][];
  impactAssessment: ImpactAssessment;
  mitigationRecommendations: MitigationRecommendation[];
  forecast?: ForecastSummary;
  confidence: number;
}

export type WeatherRiskType = 'ice' | 'high_wind' | 'extreme_temperature' | 'heavy_precipitation' | 'general';

export type ImpactRating = 'negligible' | 'minor' | 'moderate' | 'major' | 'significant' | 'severe' | 'extreme';

export type RecommendationPriority = 'low' | 'medium' | 'high' | 'critical';

export interface WeatherSummary {
  avgTemperatureC: number;
  minTemperatureC: number;
  maxTemperatureC: number;
  avgHumidityPercent: number;
  avgWindSpeedMs: number;
  maxWindSpeedMs: number;
  avgWindGustMs?: number;
  dominantWindDirection: number;
  totalPrecipitationMm: number;
  precipitationType: PrecipitationType;
  minVisibilityM?: number;
  daysWithIceRisk: number;
  daysWithHighWind: number;
  daysWithExtremeTemp: number;
  daysWithPrecipitation: number;
}

export interface WeatherAlert {
  id: UUID;
  towerId?: UUID;
  alertType: WeatherAlertType;
  severity: AlertSeverity;
  timestamp: string;
  title: string;
  headline: string;
  message: string;
  description: string;
  data?: Record<string, any>;
  effectiveStart: string;
  effectiveEnd: string;
  expiresAt?: string;
  affectedArea: string;
  responseType: ResponseType;
  certainty: Certainty;
  urgency: Urgency;
  source: string;
  createdAt: string;
}

export type WeatherAlertType = 'wind_warning' | 'ice_warning' | 'ice_storm_warning' | 'blizzard_warning' | 'freezing_rain_warning' | 'extreme_cold_warning' | 'thunderstorm_warning' | 'heavy_snow_warning' | 'avalanche_warning' | 'frost_warning' | 'dense_fog_warning' | 'high_wind_warning';

export type ResponseType = 'monitor' | 'prepare' | 'evacuate' | 'shelter' | 'execute_shutdown';

export type Certainty = 'observed' | 'likely' | 'possible' | 'unlikely' | 'unknown';

export type Urgency = 'immediate' | 'expected' | 'future' | 'past' | 'unknown';

export interface ImpactAssessment {
  operationalImpact: string;
  structuralImpact: string;
  maintenanceImpact: string;
  estimatedCostIncreasePercent: number;
  estimatedDowntimeHours: number;
  passengerImpact: string;
}

export interface MitigationRecommendation {
  riskType: WeatherRiskType;
  priority: RecommendationPriority;
  action: string;
  estimatedCost: number;
  effectiveness: number;
}

export interface ForecastSummary {
  forecastPeriodDays: number;
  forecastTime: string;
  avgTemperatureC: number;
  minTemperatureC: number;
  maxTemperatureC: number;
  avgWindSpeedMs: number;
  maxWindSpeedMs: number;
  precipitationProbability: number;
  expectedIceDays: number;
  expectedHighWindDays: number;
  expectedStormDays: number;
  overallForecastRisk: RiskLevel;
  summaryText: string;
}

export interface WeatherForecast {
  id: UUID;
  towerId?: UUID;
  forecastTime: string;
  forecastHours: number;
  hourlyForecast: HourlyForecast[];
  source: string;
  createdAt: string;
}

export interface HourlyForecast {
  timestamp: string;
  temperatureC: number;
  humidityPercent: number;
  windSpeedMs: number;
  windDirectionDeg: number;
  windGustMs?: number;
  precipitationProbabilityPercent: number;
  precipitationMm: number;
  precipitationType: PrecipitationType;
  cloudCoverPercent: number;
}

export interface VideoVerificationRequest {
  id: UUID;
  towerId: UUID;
  cameraId: UUID;
  alertId?: UUID;
  requestType: VerificationType;
  status: VerificationStatus;
  priority: Priority;
  requestedBy: UUID;
  requestedAt: string;
  description: string;
  itemsToVerify: string[];
  scheduledTime?: string;
  expiresAt?: string;
}

export type VerificationType = 'ice_presence' | 'ice_thickness' | 'structural_damage' | 'cable_condition' | 'wind_effect' | 'general_inspection' | 'alert_confirmation' | 'incident_review';

export type VerificationStatus = 'pending' | 'in_progress' | 'completed' | 'expired' | 'cancelled';

export interface VideoVerificationResult {
  id: UUID;
  requestId: UUID;
  towerId: UUID;
  cameraId: UUID;
  verifiedBy?: UUID;
  verificationMethod: VerificationMethod;
  startedAt: string;
  completedAt?: string;
  overallFindings: string[];
  aiConfidence?: number;
  humanReviewRequired: boolean;
  humanReviewed: boolean;
  reviewedBy?: UUID;
  reviewedAt?: string;
  reviewNotes?: string;
  createdAt: string;
}

export type VerificationMethod = 'ai_only' | 'human_only' | 'ai_with_human_review' | 'live_stream' | 'snapshot' | 'recorded_video';

export interface LiveStreamSession {
  id: UUID;
  cameraId: UUID;
  towerId: UUID;
  userId: UUID;
  sessionType: SessionType;
  startedAt: string;
  endedAt?: string;
  durationSeconds?: number;
  streamUrl: string;
  recordingUrl?: string;
  notes?: string;
}

export type SessionType = 'manual_review' | 'alert_response' | 'scheduled_inspection' | 'emergency_response' | 'training';

export interface HealthCheckResponse {
  status: 'healthy' | 'unhealthy' | 'degraded';
  version: string;
  uptimeSeconds: number;
  timestamp: string;
  checks?: {
    database?: HealthStatus;
    nats?: HealthStatus;
    redis?: HealthStatus;
  };
}

export type HealthStatus = 'healthy' | 'unhealthy' | 'degraded';

export interface WebSocketMessage {
  type: 'subscribe' | 'unsubscribe' | 'alert' | 'tower_status' | 'sensor_data' | 'heartbeat' | 'error';
  data?: any;
  channel?: string;
  towerId?: UUID;
}

export interface SensorDataMessage {
  towerId: UUID;
  dataType: string;
  data: any;
  timestamp: string;
}

export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
}

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: ApiError;
  timestamp: string;
}

export interface ApiError {
  code: number;
  message: string;
  details?: any;
}
