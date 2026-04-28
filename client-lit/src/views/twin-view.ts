import { LitElement, html, css } from 'lit';
import { customElement, state } from 'lit/decorators.js';
import * as THREE from 'three';

@customElement('vlt-twin-view')
export class TwinView extends LitElement {
  static styles = css`
    :host { display: block; height: 100%; position: relative; }
    .placeholder {
      display: flex; align-items: center; justify-content: center;
      height: 100%; color: var(--muted); font-size: 14px; flex-direction: column; gap: 8px;
    }
    canvas { display: block; width: 100%; height: 100%; }
  `;

  @state() private _ready = false;
  private _renderer?: THREE.WebGLRenderer;
  private _scene?: THREE.Scene;
  private _camera?: THREE.PerspectiveCamera;
  private _frameId?: number;

  connectedCallback() {
    super.connectedCallback();
    this._ready = true;
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    if (this._frameId) cancelAnimationFrame(this._frameId);
    this._renderer?.dispose();
  }

  firstUpdated() {
    this._initThree();
  }

  private _initThree() {
    const host = this.shadowRoot?.querySelector('.three-container') as HTMLElement;
    if (!host) return;
    const rect = host.getBoundingClientRect();
    if (rect.width === 0 || rect.height === 0) return;

    const scene = new THREE.Scene();
    scene.background = new THREE.Color('#09090b');
    this._scene = scene;

    const camera = new THREE.PerspectiveCamera(60, rect.width / rect.height, 0.1, 100);
    camera.position.set(5, 5, 5);
    camera.lookAt(0, 0, 0);
    this._camera = camera;

    const renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setSize(rect.width, rect.height);
    renderer.setPixelRatio(devicePixelRatio);
    host.appendChild(renderer.domElement);
    this._renderer = renderer;

    // Lights
    scene.add(new THREE.AmbientLight(0xffffff, 0.4));
    const dir = new THREE.DirectionalLight(0xffffff, 0.8);
    dir.position.set(5, 10, 7);
    scene.add(dir);

    // Ground
    const ground = new THREE.Mesh(
      new THREE.PlaneGeometry(20, 20),
      new THREE.MeshStandardMaterial({ color: '#0b1210' })
    );
    ground.rotation.x = -Math.PI / 2;
    ground.position.y = -0.01;
    scene.add(ground);

    // Grid
    scene.add(new THREE.GridHelper(20, 20, '#1a3324', '#152820'));

    // Sample equipment
    this._addBox([0, 0.5, 0], '#22c55e', 'Inverter');
    this._addBox([3, 0.5, 2], '#eab308', 'Solar Panel');
    this._addBox([-2, 0.5, 3], '#3b82f6', 'Gateway');

    // Orbit controls (simple mouse drag)
    let isDragging = false;
    let lastX = 0, lastY = 0;
    const canvas = renderer.domElement;
    canvas.addEventListener('mousedown', (e: MouseEvent) => { isDragging = true; lastX = e.clientX; lastY = e.clientY; });
    canvas.addEventListener('mouseup', () => { isDragging = false; });
    canvas.addEventListener('mousemove', (e: MouseEvent) => {
      if (!isDragging) return;
      const dx = e.clientX - lastX;
      const dy = e.clientY - lastY;
      lastX = e.clientX;
      lastY = e.clientY;
      const radius = Math.sqrt(camera.position.x ** 2 + camera.position.z ** 2);
      const angle = Math.atan2(camera.position.z, camera.position.x);
      const newAngle = angle - dx * 0.01;
      camera.position.x = radius * Math.cos(newAngle);
      camera.position.z = radius * Math.sin(newAngle);
      camera.position.y = Math.max(0.5, camera.position.y - dy * 0.05);
      camera.lookAt(0, 0, 0);
    });
    canvas.addEventListener('wheel', (e: WheelEvent) => {
      e.preventDefault();
      const factor = e.deltaY > 0 ? 1.1 : 0.9;
      camera.position.multiplyScalar(factor);
    });

    const animate = () => {
      this._frameId = requestAnimationFrame(animate);
      renderer.render(scene, camera);
    };
    animate();
  }

  private _addBox(pos: [number, number, number], color: string, label: string) {
    if (!this._scene) return;
    const geo = new THREE.BoxGeometry(1, 1, 1);
    const mat = new THREE.MeshStandardMaterial({ color });
    const mesh = new THREE.Mesh(geo, mat);
    mesh.position.set(...pos);
    mesh.castShadow = true;
    this._scene.add(mesh);
  }

  render() {
    return html`
      <div class="three-container" style="width:100%;height:100%">
        ${!this._ready ? html`<div class="placeholder">Loading 3D scene...</div>` : ''}
      </div>
    `;
  }
}
