import { useState } from 'react';
import { inputStyle, labelStyle, labelTextStyle, btnStyle } from './styles';

export function ConfirmDialog({ title, message, confirmLabel, onConfirm, onClose }: {
  title: string; message: string; confirmLabel?: string; onConfirm: () => void; onClose: () => void;
}) {
  return (
    <div style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,.6)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 60 }} onClick={onClose}>
      <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10, width: 380, padding: 20 }} onClick={e => e.stopPropagation()}>
        <div style={{ fontSize: 15, fontWeight: 600, marginBottom: 12 }}>{title}</div>
        <div style={{ fontSize: 13, color: 'var(--muted)', marginBottom: 20 }}>{message}</div>
        <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end' }}>
          <button onClick={onClose} style={{ padding: '7px 16px', borderRadius: 6, border: '1px solid var(--border)', background: 'transparent', color: 'var(--fg)', cursor: 'pointer', fontSize: 13 }}>Cancel</button>
          <button onClick={() => { onConfirm(); onClose(); }} style={{ padding: '7px 16px', borderRadius: 6, border: '1px solid var(--crit)', background: 'var(--crit)', color: '#fff', cursor: 'pointer', fontSize: 13, fontWeight: 600 }}>{confirmLabel || 'Delete'}</button>
        </div>
      </div>
    </div>
  );
}

export function PromptDialog({ title, label, placeholder, initialValue, okLabel, onSubmit, onClose }: {
  title: string; label: string; placeholder?: string; initialValue?: string; okLabel?: string;
  onSubmit: (value: string) => void; onClose: () => void;
}) {
  const [value, setValue] = useState(initialValue || '');
  return (
    <div style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,.6)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 60 }} onClick={onClose}>
      <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10, width: 380, padding: 20 }} onClick={e => e.stopPropagation()}>
        <div style={{ fontSize: 15, fontWeight: 600, marginBottom: 16 }}>{title}</div>
        <label style={labelStyle}>
          <span style={labelTextStyle}>{label}</span>
          <input value={value} onChange={e => setValue(e.target.value)} placeholder={placeholder} style={inputStyle} autoFocus onKeyDown={e => { if (e.key === 'Enter' && value.trim()) { onSubmit(value.trim()); onClose(); } }} />
        </label>
        <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end', marginTop: 16 }}>
          <button onClick={onClose} style={{ padding: '7px 16px', borderRadius: 6, border: '1px solid var(--border)', background: 'transparent', color: 'var(--fg)', cursor: 'pointer', fontSize: 13 }}>Cancel</button>
          <button onClick={() => { if (value.trim()) { onSubmit(value.trim()); onClose(); } }} style={{ padding: '7px 16px', borderRadius: 6, border: '1px solid var(--cyan)', background: 'transparent', color: 'var(--cyan)', cursor: 'pointer', fontSize: 13, fontWeight: 600 }}>{okLabel || 'OK'}</button>
        </div>
      </div>
    </div>
  );
}
