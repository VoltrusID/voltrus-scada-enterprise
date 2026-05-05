import React from 'react'
import { registerWidget, WIDGET_MANIFESTS } from './registry'
import type { WidgetProps } from './types'

const LabelWidget: React.FC<WidgetProps> = (props) => {
  const { instance, sensor, selected } = props
  const config = instance.config ?? {}
  const fontSize = config.fontSize ?? 14

  const text = config.text || sensor?.name || 'Label'

  const containerStyle: React.CSSProperties = {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    width: '100%',
    height: '100%',
    padding: '4px 8px',
    boxSizing: 'border-box',
    overflow: 'hidden',
    border: selected ? '1px dashed var(--accent)' : 'none',
    borderRadius: selected ? 4 : 0,
  }

  const textStyle: React.CSSProperties = {
    fontSize,
    color: 'var(--fg)',
    whiteSpace: 'nowrap',
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    textAlign: 'center',
    lineHeight: 1.3,
    userSelect: 'none',
  }

  return (
    <div style={containerStyle}>
      <span style={textStyle}>{text}</span>
    </div>
  )
}

registerWidget({
  manifest: WIDGET_MANIFESTS.label,
  component: LabelWidget,
})

export default LabelWidget
