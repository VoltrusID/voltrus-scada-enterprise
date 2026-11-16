# Scheduled reports

Daily, weekly or monthly reports of tag statistics and alarms, rendered as PDF
and HTML, saved on the server and emailed with the PDF attached. Licence:
Integrator or Enterprise (`pdf_reporting`). Code: `enterprise/src/reports/`.

## What a report holds

* Period: the last 24 hours (`daily`), 7 days (`weekly`) or 30 days
  (`monthly`) before the run, in UTC.
* Device metrics: for every tag of the chosen devices (all by default) with
  history in the period, min, max, average and a trend line.
* Alarm log: alarms raised in the period (newest 500): time, severity, source,
  message, value.

Formats: PDF (A4, built in, no external tools) and HTML. There are no SMS
reports, and no CSV or Excel report format (use the core's data export for
raw values).

## When it runs

The scheduler checks every 60 seconds on the active node. A report is due when
its period has passed since its last scheduled run (`last_generated`); a new
report has no last run, so **it runs within a minute of being saved** and then
every 24 h, 7 d or 30 d after that. There is no time-of-day setting. A run
counts even if mail fails (the files are saved and the error is logged), so a
broken relay is not retried every minute.

Files: `reports/<report id>/report_<date>_<schedule>.pdf` and `.html` next to
the database file (`db_path`). Remove old ones with your usual housekeeping;
the server does not prune them.

## Set up

1. Licence: **More actions (⋯) › Data Manager › Admin › License** shows
   Integrator or Enterprise.
2. Mail server: **More actions (⋯) › System › SMTP**: host, port, user,
   password, from address and name, encryption, then **Save Configuration**, and **Test Connection › Send Test**
   to check. This writes the `smtp:` block of `config.yaml`; the scheduler picks
   the new settings up without a restart. In `config.yaml` directly:

   ```yaml
   smtp:
     host: smtp.example.com
     port: 587
     username: voltrus@example.com
     password: "<app password>"
     from_address: voltrus@example.com
     from_name: Voltrus Plant A
     encryption: starttls    # starttls | implicittls | none
   ```

3. Report: **Data Manager › Insights › Reports › + Add Report**:
   name, schedule (daily, weekly, monthly), recipients
   (one address per line), enabled. Save. API:

   ```sh
   curl -b jar -H 'Content-Type: application/json' \
     -d '{"name":"Plant A daily","schedule":"daily","email_recipients":["ops@example.com"]}' \
     https://scada.example.com/api/v1/reports
   ```

   `email_recipients` and `device_ids` take a JSON array or the same array as
   JSON text (what the SDKs send); `device_ids` defaults to `"all"`.

## Check it works

1. Within a minute of saving, the server log shows
   `Generating report '<name>'` then
   `Report '<name>': saved <path>.pdf, emailed to N recipient(s)`.
2. The PDF and HTML are in `reports/<id>/` next to the database.
3. Each recipient gets `[Voltrus] <name> (Daily report)` with a short summary
   and the PDF attached.
4. The **Reports** list shows a **Last Generated** time.
5. **Preview** opens the HTML and **PDF** downloads the PDF of the current
   period (`POST /api/v1/reports/<id>/generate`, add `?format=pdf`); neither
   sends mail nor moves the schedule.

Verified on 2026-11-16: SMTP set through the API after start, a daily report
created with the web form's request shape ran 50 s later; two emails with the
PDF reached a local SMTP catcher; the PDF passed `qpdf --check` and
`pdftotext` showed the metrics and two alarms.

## Troubleshooting

| Log line or symptom | Cause and fix |
| --- | --- |
| `saved … but SMTP is not configured` | Set **System › SMTP**. The next run (24 h later for daily) mails; or delete and re-add the report to run it now. |
| `emailed to 0 of N; failed: …SMTP…` | The relay refused the login or the sender; use **System › SMTP › Send Test** and fix host, port, encryption or credentials. |
| `403 PDF reporting requires an Integrator or Enterprise licence` | No licence with reports. |
| Report empty ("No telemetry data") | The chosen devices have no history in the period, or `device_ids` names devices that do not exist. |
| Reports stop after a failover | Only the active node of a hot standby pair runs reports; check `GET /api/v1/ha/status` on both nodes. |
| Nothing in the native console | Reports are managed in the web client only. |

## Fixed in the audit of 2026-11-16

Before: the scheduler checked once an hour, sent no PDF (an alarm-style email
without attachment; the report itself was never saved), did not start at all
when SMTP was configured after start, the web form's create and edit requests
were rejected (arrays where the server wanted JSON text), and a manual
"Generate" reset the schedule.
