import { component$, useTask$, useSignal, $ } from '@builder.io/qwik';
import L from 'leaflet';
import 'leaflet/dist/leaflet.css';
import type { Tower } from '~/types';

interface TowerMapProps {
  towers: Tower[];
  height?: string;
  onTowerClick$?: (tower: Tower) => void;
}

export default component$<TowerMapProps>(({
  towers,
  height = '400px',
  onTowerClick$,
}) => {
  const mapRef = useSignal<HTMLDivElement>();
  const mapInstance = useSignal<L.Map | null>(null);
  const markersRef = useSignal<L.Marker[]>([]);

  const getTowerColor = (status: string) => {
    switch (status) {
      case 'online': return '#22c55e';
      case 'warning': return '#f59e0b';
      case 'danger': return '#ef4444';
      default: return '#9ca3af';
    }
  };

  const createTowerIcon = (color: string) => {
    return L.divIcon({
      className: 'custom-marker',
      html: `<div style="
        width: 24px;
        height: 24px;
        background: ${color};
        border: 3px solid white;
        border-radius: 50%;
        box-shadow: 0 2px 8px rgba(0,0,0,0.3);
        position: relative;
      ">
        <div style="
          position: absolute;
          top: 50%;
          left: 50%;
          transform: translate(-50%, -50%);
          width: 8px;
          height: 8px;
          background: white;
          border-radius: 50%;
        "></div>
      </div>`,
      iconSize: [24, 24],
      iconAnchor: [12, 12],
    });
  };

  const initMap = $(() => {
    if (!mapRef.value) return;

    if (mapInstance.value) {
      mapInstance.value.remove();
    }

    const map = L.map(mapRef.value).setView([30.5, 103.0], 12);

    L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
      attribution: '&copy; OpenStreetMap contributors',
    }).addTo(map);

    mapInstance.value = map;

    const markers: L.Marker[] = [];
    towers.forEach((tower) => {
      const color = getTowerColor(tower.status);
      const icon = createTowerIcon(color);
      
      const marker = L.marker([tower.latitude, tower.longitude], { icon })
        .bindPopup(`
          <div style="min-width: 180px;">
            <h3 style="margin: 0 0 8px 0; font-size: 14px; font-weight: 600;">${tower.name}</h3>
            <p style="margin: 4px 0; font-size: 12px; color: #6b7280;">
              <strong>编号:</strong> ${tower.code}
            </p>
            <p style="margin: 4px 0; font-size: 12px; color: #6b7280;">
              <strong>状态:</strong> 
              <span style="color: ${color}; font-weight: 500;">
                ${tower.status === 'online' ? '正常' : tower.status === 'warning' ? '告警' : tower.status === 'danger' ? '危险' : '离线'}
              </span>
            </p>
            <p style="margin: 4px 0; font-size: 12px; color: #6b7280;">
              <strong>海拔:</strong> ${tower.elevation}m
            </p>
          </div>
        `)
        .addTo(map);

      if (onTowerClick$) {
        marker.on('click', () => onTowerClick$(tower));
      }

      markers.push(marker);
    });

    markersRef.value = markers;

    if (markers.length > 0) {
      const group = L.featureGroup(markers);
      map.fitBounds(group.getBounds().pad(0.1));
    }

    const handleResize = () => map.invalidateSize();
    window.addEventListener('resize', handleResize);

    return () => {
      window.removeEventListener('resize', handleResize);
      map.remove();
    };
  });

  useTask$(() => {
    initMap();
  });

  return <div ref={mapRef} style={{ height, width: '100%', borderRadius: '0.5rem' }}></div>;
});
