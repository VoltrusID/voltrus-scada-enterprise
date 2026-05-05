import type { WidgetInstance, ConnectionInstance, ScadaWidgetType } from '../widgets/types'

export interface ScadaLayoutState {
  widgets: WidgetInstance[]
  connections: ConnectionInstance[]
}

type Action =
  | { type: 'ADD_WIDGET'; widget: WidgetInstance }
  | { type: 'REMOVE_WIDGET'; id: string }
  | { type: 'UPDATE_WIDGET'; id: string; updates: Partial<WidgetInstance> }
  | { type: 'MOVE_WIDGET'; id: string; x: number; y: number }
  | { type: 'RESIZE_WIDGET'; id: string; x: number; y: number; width: number; height: number }
  | { type: 'ADD_CONNECTION'; connection: ConnectionInstance }
  | { type: 'REMOVE_CONNECTION'; id: string }
  | { type: 'LOAD_LAYOUT'; widgets: WidgetInstance[]; connections: ConnectionInstance[] }
  | { type: 'CLEAR' }

export function layoutReducer(state: ScadaLayoutState, action: Action): ScadaLayoutState {
  switch (action.type) {
    case 'ADD_WIDGET':
      return { ...state, widgets: [...state.widgets, action.widget] }

    case 'REMOVE_WIDGET': {
      const id = action.id
      return {
        widgets: state.widgets.filter(w => w.id !== id),
        connections: state.connections.filter(c => c.fromWidgetId !== id && c.toWidgetId !== id),
      }
    }

    case 'UPDATE_WIDGET':
      return {
        ...state,
        widgets: state.widgets.map(w =>
          w.id === action.id ? { ...w, ...action.updates } : w,
        ),
      }

    case 'MOVE_WIDGET':
      return {
        ...state,
        widgets: state.widgets.map(w =>
          w.id === action.id ? { ...w, x: action.x, y: action.y } : w,
        ),
      }

    case 'RESIZE_WIDGET':
      return {
        ...state,
        widgets: state.widgets.map(w =>
          w.id === action.id
            ? { ...w, x: action.x, y: action.y, width: action.width, height: action.height }
            : w,
        ),
      }

    case 'ADD_CONNECTION':
      return { ...state, connections: [...state.connections, action.connection] }

    case 'REMOVE_CONNECTION':
      return {
        ...state,
        connections: state.connections.filter(c => c.id !== action.id),
      }

    case 'LOAD_LAYOUT':
      return { widgets: action.widgets, connections: action.connections }

    case 'CLEAR':
      return { widgets: [], connections: [] }

    default:
      return state
  }
}

export function createWidget(
  type: ScadaWidgetType,
  sensorId: number | null,
  x: number,
  y: number,
  defaultWidth: number,
  defaultHeight: number,
): WidgetInstance {
  return {
    id: `w_${Date.now()}_${Math.random().toString(36).slice(2, 7)}`,
    type,
    sensorId,
    x,
    y,
    width: defaultWidth,
    height: defaultHeight,
    config: {},
  }
}

export function createConnection(
  fromId: string,
  toId: string,
  connectionType: 'power' | 'data' | 'flow',
): ConnectionInstance {
  return {
    id: `c_${Date.now()}_${Math.random().toString(36).slice(2, 7)}`,
    fromWidgetId: fromId,
    toWidgetId: toId,
    connectionType,
    waypoints: [],
  }
}
