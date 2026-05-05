import { LitElement } from 'lit';
import './edit-panel.js';
export declare class DataManager extends LitElement {
    static styles: import("lit").CSSResult;
    isAdmin: boolean;
    private _tab;
    private _sensors;
    private _editingId;
    private _filter;
    private _modbusDevices;
    private _modbusFilter;
    private _users;
    private _scanRange;
    private _scanPort;
    private _scanning;
    private _scanResults;
    private _showScanResults;
    private _userFilter;
    connectedCallback(): void;
    disconnectedCallback(): void;
    private _onKey;
    private _loadSensors;
    private _loadModbus;
    private _loadUsers;
    private _saveSensor;
    private _deleteSensor;
    private _duplicateSensor;
    private _deleteUser;
    private _deleteModbus;
    private _scanNetwork;
    private _close;
    private _sourcesTab;
    private _modbusTab;
    private _usersTab;
    render(): import("lit-html").TemplateResult<1>;
}
//# sourceMappingURL=data-manager.d.ts.map