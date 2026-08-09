# CRM SaaS — Phase 1 Product & UX Specification

## 1. Purpose

This document defines the Phase 1 scope, user experience, functional requirements, core data model, role permissions, and implementation guidance for a multi-tenant CRM SaaS focused on customer registration, lead distribution, partner processing, and sales outcome tracking.

Phase 1 is intentionally focused. It is **not** intended to replicate a full enterprise CRM such as Salesforce or HubSpot. The initial product should function as a clean, reliable **Lead Distribution CRM / Partner Lead Management Platform** with an architecture that can evolve into a broader CRM in later phases.

The core business flow is:

```text
SALES REPRESENTATIVE

Customer Conversation
        ↓
Register Customer
        ↓
Select Product Categories
        ↓
Submit
        ↓

SYSTEM

Customer
        ↓
Lead(s)
        ↓

ADMINISTRATOR

Assign Lead
        ↓
One or More Partner Companies
        ↓

PARTNER COMPANY

New
 ↓
Contacted
 ↓
In Progress
 ├───────────────┐
 ↓               ↓
Sale           No Sale
```

---

## 2. Phase 1 Goals

The system must enable:

1. Administrators to manage Sales Representatives, Partner Companies, and Product Categories.
2. Sales Representatives to register customers and submit product-interest leads during customer conversations.
3. Administrators to manually distribute leads to one or more eligible Partner Companies.
4. Partner Companies to securely view only the leads assigned to them.
5. Partner Companies to update lead processing status and record whether a lead resulted in a sale.
6. Administrators to monitor customers, leads, assignments, representatives, partners, and registered sales.
7. A simple operational dashboard showing Phase 1 performance metrics.
8. A data model that supports later API integrations, automatic routing, richer sales workflows, and reporting without major restructuring.

---

# 3. Product Principles

## 3.1 Customer Is Not a Lead

A Customer represents the person whose information is registered.

A Lead represents a specific product or service interest from that Customer.

Example:

```text
Customer: John Smith

Interests:
- Personal Loan
- Insurance
- Solar

Generated records:

Lead #001 — Personal Loan
Lead #002 — Insurance
Lead #003 — Solar
```

This separation is mandatory because one Customer may have multiple interests that are processed independently.

---

## 3.2 Lead Is Not a Partner Assignment

A Lead may be distributed to one or more Partner Companies.

Therefore, the Lead must not contain a single `partner_id`.

Instead:

```text
Lead
  │
  ├── LeadAssignment → Partner A
  ├── LeadAssignment → Partner B
  └── LeadAssignment → Partner C
```

Each Partner Company must be able to independently process the same Lead.

Example:

```text
Lead: Personal Loan / John Smith

Partner A → No Sale
Partner B → Sale
Partner C → Contacted
```

The processing status belongs primarily to the `LeadAssignment`, not the Lead itself.

---

## 3.3 Phase 1 Should Remain Operationally Simple

Do not introduce unnecessary enterprise CRM complexity.

Phase 1 does not require:

- marketing automation,
- AI agents,
- advanced CRM pipelines,
- quoting,
- proposals,
- customer support tickets,
- complex BI,
- automatic partner routing,
- third-party API integrations,
- workflow builders,
- custom objects,
- advanced forecasting.

The architecture should permit these later, but Phase 1 UX should remain simple.

---

# 4. User Roles

The application has three primary role types.

```text
Administrator
Sales Representative
Partner User
```

The application should remain a single SaaS platform rather than three independent applications.

After authentication, the visible navigation and permitted actions should depend on the authenticated user's role.

---

# 5. Administrator Portal

## 5.1 Administrator Navigation

Recommended sidebar:

```text
Dashboard
Customers
Leads
Sales Representatives
Partners
Product Categories
Reports
Settings
```

`Reports` and `Settings` may remain minimal in Phase 1.

---

# 6. Administrator Dashboard

The Phase 1 dashboard should prioritize operational clarity rather than complex analytics.

Required metrics:

- Total Customers
- Total Leads
- Leads per Sales Representative
- Leads per Partner Company
- Sales registered by Partner Companies

Suggested layout:

```text
┌────────────────────────────────────────────────────────┐
│ Dashboard                                              │
│                                                        │
│ [1,248 Customers] [1,531 Leads] [284 Sales]           │
├────────────────────────────────────────────────────────┤
│                                                        │
│ Leads by Sales Representative                          │
│                                                        │
│ Eko              182                                   │
│ Maya             145                                   │
│ Dimas            127                                   │
│                                                        │
├────────────────────────────────────────────────────────┤
│                                                        │
│ Leads by Partner                                       │
│                                                        │
│ Partner A         320                                  │
│ Partner B         210                                  │
│ Partner C         145                                  │
│                                                        │
├────────────────────────────────────────────────────────┤
│                                                        │
│ Sales Registered by Partners                           │
│                                                        │
│ Partner A          62                                  │
│ Partner B          41                                  │
└────────────────────────────────────────────────────────┘
```

Phase 1 should favor summary cards, tables, and simple charts.

Avoid building a full analytics platform.

---

# 7. Sales Representative Management

## 7.1 Sales Representative List

Route suggestion:

```text
/admin/sales-representatives
```

Suggested page:

```text
Sales Representatives                         + Add Representative

Name             Email               Customers    Leads    Status
------------------------------------------------------------------
Eko              eko@...             145          172      Active
Maya             maya@...            103          119      Active
Dimas            dimas@...            89           94      Inactive
```

Recommended capabilities:

- Create Sales Representative
- Edit Sales Representative
- Activate / deactivate account
- Reset password or initiate password reset
- View created Customers
- View submitted Leads

---

## 7.2 Sales Representative Detail

Route suggestion:

```text
/admin/sales-representatives/{id}
```

Suggested layout:

```text
EKO SUPRAPTO

Status: Active

Email
Phone
Created At

────────────────────────────

Customers Created
145

Leads Submitted
172

Sales Resulted
38
```

Recommended tabs:

```text
Overview
Customers
Leads
```

---

# 8. Partner Company Management

## 8.1 Partner Company List

Route suggestion:

```text
/admin/partners
```

Suggested page:

```text
Partner Companies                              + Add Partner

Company         Categories            Leads    Sales    Status
----------------------------------------------------------------
Acme Finance    Loan, Insurance       320      62       Active
XYZ Telecom     Internet              210      41       Active
ABC Energy      Solar                 145      28       Active
```

Admin capabilities:

- Create Partner Company
- Edit Partner Company
- Activate / deactivate Partner Company
- Manage Partner users
- Assign Product Categories
- View assigned Leads
- View registered Sales
- View basic performance metrics

---

## 8.2 Partner Company Detail

Route suggestion:

```text
/admin/partners/{id}
```

Suggested layout:

```text
ACME FINANCE

Status: Active

Contact Person
Email
Phone
Address

────────────────────────────

PRODUCT CATEGORIES

✓ Personal Loan
✓ Business Loan
✓ Insurance

[Manage Categories]

────────────────────────────

PERFORMANCE

Assigned Leads        320
New                    43
Contacted              40
In Progress            74
Sales                  62
No Sale               101
```

Recommended tabs:

```text
Overview
Users
Categories
Leads
Sales
```

---

# 9. Product Category Management

## 9.1 Product Category List

Route suggestion:

```text
/admin/product-categories
```

Suggested page:

```text
Product Categories                         + Add Category

Category                Partners    Leads
--------------------------------------------
Personal Loan           4           362
Home Insurance          3           241
Solar Panel             2           119
Internet Service        5           412
```

Capabilities:

- Create category
- Rename category
- Activate / deactivate category
- Assign Partner Companies
- View Lead count
- View Partner count

Prefer deactivation over hard deletion when a category has historical records.

---

## 9.2 Product Category Detail

Route suggestion:

```text
/admin/product-categories/{id}
```

Suggested UI:

```text
PERSONAL LOAN

Assigned Partners

☑ Acme Finance
☑ ABC Finance
☐ XYZ Finance

[Save]
```

The relationship between Product Categories and Partner Companies should be many-to-many.

This relationship becomes the foundation for future automatic routing.

---

# 10. Sales Representative Portal

## 10.1 Navigation

Recommended sidebar:

```text
Dashboard
Customers
New Customer
My Leads
```

The portal must optimize for speed because representatives may enter information while actively speaking with customers.

---

# 11. Sales Representative Dashboard

Keep the Phase 1 dashboard simple.

Suggested metrics:

```text
My Customers
My Leads
Sales Resulted
Recent Leads
```

Example:

```text
[145 Customers] [172 Leads] [38 Sales]

Recent Leads
-------------------------------------------------
John Smith       Personal Loan      Assigned
Sarah Doe        Insurance          In Progress
David Lee        Solar              Sale
```

No advanced forecasting is required.

---

# 12. Register Customer UX

## 12.1 Route

```text
/sales/customers/new
```

This is one of the most important pages in Phase 1.

The form must support fast entry with minimal navigation.

Suggested layout:

```text
NEW CUSTOMER

Customer Information

First Name *
[________________________]

Last Name *
[________________________]

Phone *
[________________________]

Email
[________________________]

Address
[________________________]

City
[________________________]

Postcode
[________________________]
```

Then:

```text
WHAT IS THE CUSTOMER INTERESTED IN?

☑ Personal Loan
☐ Home Loan
☑ Insurance
☐ Solar
☐ Internet

                   [Save Customer & Submit Lead]
```

Product Categories must support multi-selection.

---

## 12.2 Submission Behavior

The UX should expose one primary action:

```text
Save Customer & Submit Lead
```

Do not force the representative through a multi-step workflow such as:

```text
Create Customer
↓
Save
↓
Open Customer
↓
Create Lead
↓
Select Category
↓
Save
```

Internally, the backend may create several records in one transaction.

Example:

```text
Customer: John Smith

Selected categories:
- Personal Loan
- Insurance

System creates:

Customer #C1001
Lead #L1001 — Personal Loan
Lead #L1002 — Insurance
```

The user should experience this as a single workflow.

---

# 13. Customer Management

## 13.1 Customer Entity

Recommended conceptual fields:

```text
Customer
- id
- tenant_id
- first_name
- last_name
- phone
- email
- address
- city
- postcode
- created_by_sales_rep_id
- created_at
- updated_at
```

Additional fields may be added according to client requirements, but avoid unnecessary complexity in Phase 1.

---

## 13.2 Customer List

Administrator:

```text
/admin/customers
```

Sales Representative:

```text
/sales/customers
```

Administrator sees all Customers within the tenant.

Sales Representatives should normally see Customers they created unless business rules explicitly require broader visibility.

Suggested table:

```text
Customer       Phone         Sales Rep    Leads    Created
------------------------------------------------------------
John Smith     +44 ...       Eko          2        Today
Sarah Doe      +44 ...       Maya         1        Yesterday
```

Recommended features:

- Search
- Filter by Sales Representative
- Filter by creation date
- Sort
- Pagination

---

## 13.3 Customer Detail

Suggested route:

```text
/admin/customers/{id}
```

or:

```text
/sales/customers/{id}
```

Suggested layout:

```text
JOHN SMITH

Phone
Email
Address
City
Postcode

Created By
Eko Suprapto

Created
9 Aug 2026

────────────────────────────

LEADS

Personal Loan        Acme Finance       In Progress
Insurance            ABC Insurance     Sale
```

Recommended tabs:

```text
Overview
Leads
Activity
```

Activity can remain simple in Phase 1.

---

# 14. Lead Model

A Lead represents one Customer interest in one Product Category.

Recommended conceptual model:

```text
Lead
- id
- tenant_id
- customer_id
- product_category_id
- created_by_sales_rep_id
- created_at
- updated_at
```

A Lead does not directly own a single Partner Company.

Partner distribution is represented through `LeadAssignment`.

---

# 15. Administrator Lead Management

## 15.1 Lead List

Route suggestion:

```text
/admin/leads
```

Suggested layout:

```text
LEADS

Customer       Category         Sales Rep    Partners      Status
-----------------------------------------------------------------
John Smith     Personal Loan    Eko          —             Unassigned
Sarah Doe      Insurance        Maya         ABC Insurance Assigned
David Lee      Solar            Eko          Green Power   In Progress
```

Recommended filters:

- Product Category
- Sales Representative
- Partner Company
- Assignment status
- Created date
- Sale / No Sale
- Search by Customer name, phone, or email

---

# 16. Manual Lead Distribution

## 16.1 Lead Detail

Route suggestion:

```text
/admin/leads/{id}
```

Suggested layout:

```text
JOHN SMITH
Personal Loan

Created By
Eko

Customer Details
Phone
Email
Address

─────────────────────────

ASSIGN PARTNERS

Eligible Partners

☐ Acme Finance
☐ ABC Finance
☐ Loan Corp

[Assign]
```

The Administrator may assign one Lead to one or more Partner Companies.

Only eligible Partners assigned to the Lead's Product Category should normally appear.

If the business later allows category overrides, that can be introduced as a controlled Administrator capability.

---

# 17. Lead Assignment Entity

`LeadAssignment` is a first-class domain entity.

Recommended conceptual model:

```text
LeadAssignment
- id
- tenant_id
- lead_id
- partner_company_id
- assigned_by_user_id
- assigned_at
- status
- updated_at
```

Recommended status values:

```text
NEW
CONTACTED
IN_PROGRESS
SALE
NO_SALE
```

Optional later statuses may include:

```text
UNREACHABLE
DECLINED
DUPLICATE
EXPIRED
```

Do not over-expand the Phase 1 status model unless required.

---

# 18. Lead Assignment Status Flow

Recommended Phase 1 lifecycle:

```text
NEW
 ↓
CONTACTED
 ↓
IN_PROGRESS
 ├───────────────┐
 ↓               ↓
SALE           NO_SALE
```

The system may allow direct transitions if needed operationally.

For example:

```text
NEW → NO_SALE
CONTACTED → SALE
```

Avoid enforcing an unnecessarily rigid workflow unless required by the client.

---

# 19. Partner Portal

## 19.1 Partner Navigation

Recommended sidebar:

```text
Dashboard
Leads
Sales
```

Partner users must only access data belonging to their own Partner Company.

This must be enforced server-side.

Never rely only on frontend filtering.

---

# 20. Partner Dashboard

Suggested Phase 1 dashboard:

```text
[120 Assigned Leads]
[23 New]
[32 In Progress]
[18 Sales]
[41 No Sale]
```

Optional section:

```text
Recent Leads
-------------------------------------------------
John Smith       Personal Loan       New
Sarah Doe        Personal Loan       Contacted
David Lee        Business Loan       In Progress
```

---

# 21. Partner Lead List

Route suggestion:

```text
/partner/leads
```

Suggested layout:

```text
MY LEADS

Customer       Category       Assigned      Status
------------------------------------------------------
John Smith     Personal Loan  Today         New
Sarah Doe      Personal Loan  Yesterday     Contacted
David Lee      Business Loan  3 days ago    In Progress
```

Recommended filters:

- Status
- Product Category
- Assigned date
- Customer search

---

# 22. Partner Lead Detail

Route suggestion:

```text
/partner/leads/{leadAssignmentId}
```

Important: Partner routes should preferably reference the `LeadAssignment`, because that is the record the Partner is processing.

Suggested page:

```text
JOHN SMITH

Personal Loan

Status
[ New ▾ ]

─────────────────────────

CUSTOMER DETAILS

Phone
Email
Address
City
Postcode

─────────────────────────

LEAD INFORMATION

Category
Personal Loan

Created By
Eko

Assigned
9 Aug 2026

─────────────────────────

STATUS

New
Contacted
In Progress
Sale
No Sale
```

The Partner may:

- View complete Customer information for assigned Leads.
- Update the LeadAssignment status.
- Register a Sale.
- Register No Sale.
- Add optional notes if included in the implementation.

---

# 23. Sale Registration

## 23.1 Sale Action

When the Partner selects `SALE`, display a Sale registration form.

Suggested UI:

```text
REGISTER SALE

Customer
John Smith

Product Category
Personal Loan

Sale Date
[9 Aug 2026]

Sale Value
[____________]

Reference / Notes
[________________________]

                  [Register Sale]
```

The client explicitly requires manual registration of whether a Lead resulted in a Sale.

Recommended Phase 1 fields:

```text
Sale
- id
- tenant_id
- lead_assignment_id
- sale_date
- sale_value (optional)
- reference (optional)
- notes (optional)
- registered_by_user_id
- created_at
- updated_at
```

`Sale Value` should be optional unless required by the client.

It is recommended to include it now because it enables future revenue reporting without requiring a major data model change.

---

# 24. No Sale

When a Partner marks a LeadAssignment as `NO_SALE`, optionally allow a reason.

Suggested Phase 1 reasons:

```text
Not Interested
Not Eligible
Unable to Contact
Purchased Elsewhere
Duplicate
Other
```

This field may remain optional.

Recommended model extension:

```text
LeadAssignment
- no_sale_reason
- no_sale_notes
```

---

# 25. Reports

Phase 1 reports can remain simple and table-oriented.

Recommended reports:

```text
Customers by Sales Representative
Leads by Sales Representative
Leads by Product Category
Leads by Partner Company
Sales by Partner Company
Lead Status by Partner
```

Advanced BI is explicitly out of scope.

---

# 26. Core Data Model

The core Phase 1 entities are:

```text
Tenant

User
├── Administrator
├── Sales Representative User
└── Partner User

SalesRepresentative

PartnerCompany

ProductCategory

PartnerProductCategory

Customer

Lead

LeadAssignment

Sale
```

---

# 27. Recommended Relationships

```text
Tenant
 │
 ├── Users
 ├── SalesRepresentatives
 ├── PartnerCompanies
 ├── ProductCategories
 ├── Customers
 ├── Leads
 ├── LeadAssignments
 └── Sales
```

Sales flow:

```text
Sales Representative
       │
       │ creates
       ▼
    Customer
       │
       │ requests
       ▼
      Lead
       │
       │ belongs to
       ▼
 Product Category
       │
       │ distributed through
       ▼
 Lead Assignment
       │
       ▼
 Partner Company
       │
       │ optionally produces
       ▼
      Sale
```

Partner-category relationship:

```text
PartnerCompany
      │
      │ many-to-many
      ▼
ProductCategory
```

Lead-partner relationship:

```text
Lead
      │
      │ many-to-many through LeadAssignment
      ▼
PartnerCompany
```

---

# 28. Recommended Database-Level Constraints

Important recommended constraints:

```text
Customer.tenant_id NOT NULL
Lead.tenant_id NOT NULL
Lead.customer_id NOT NULL
Lead.product_category_id NOT NULL

LeadAssignment.tenant_id NOT NULL
LeadAssignment.lead_id NOT NULL
LeadAssignment.partner_company_id NOT NULL

UNIQUE (
    tenant_id,
    lead_id,
    partner_company_id
)
```

This prevents accidental duplicate assignment of the same Lead to the same Partner.

The exact schema may vary by implementation stack.

---

# 29. SaaS / Multi-Tenant Requirements

Even if Phase 1 launches with one client organization, the system should be designed as SaaS.

Every business-domain record should be tenant-scoped.

Recommended high-level model:

```text
Tenant
  ├── Users
  ├── Customers
  ├── Leads
  ├── Partners
  ├── Categories
  ├── Assignments
  └── Sales
```

Never trust tenant identifiers supplied directly by the frontend.

Tenant context must be derived from the authenticated session/token and enforced on the backend.

---

# 30. Role & Permission Matrix

## Administrator

Can:

- Manage Sales Representatives
- Manage Partner Companies
- Manage Partner users
- Manage Product Categories
- Assign categories to Partners
- View all Customers
- View all Leads
- View creator of each Customer
- View creator of each Lead
- Assign Leads to Partners
- View all LeadAssignments
- View all Partner status updates
- View all Sales
- View Dashboard
- View Reports

---

## Sales Representative

Can:

- Log in
- View own dashboard
- Register Customer
- Edit own Customer records according to business rules
- Select one or more Product Categories
- Submit Leads
- View Customers created by self
- View Leads created by self
- View current Lead outcome/status if allowed

Cannot:

- Manage other Sales Representatives
- Manage Partners
- Manage Product Categories
- Assign Leads
- Modify Partner processing status
- View unrelated Customers unless explicitly permitted

---

## Partner User

Can:

- Log in
- View Partner dashboard
- View LeadAssignments belonging to own Partner Company
- View complete Customer details for assigned Leads
- Update own LeadAssignment status
- Register Sale
- Register No Sale
- View Sales registered by own Partner Company

Cannot:

- View other Partners' Leads
- View other Partners' assignment statuses
- View Customers without an associated assignment
- Assign Leads
- Manage Sales Representatives
- Manage global Product Categories

---

# 31. Authentication & Security

Phase 1 requires secure authentication.

Minimum expectations:

- Secure password hashing
- Session or token-based authentication
- Server-side authorization
- Tenant isolation
- Role-based access control
- Partner-company isolation
- CSRF protection where applicable
- Secure cookies where applicable
- Input validation
- Rate limiting for authentication endpoints
- Audit timestamps
- Soft delete or deactivation for critical reference records

Partner access control must be enforced by queries such as:

```text
LeadAssignment
WHERE partner_company_id = authenticated_user.partner_company_id
AND tenant_id = authenticated_user.tenant_id
```

Do not fetch global data and filter it only in the browser.

---

# 32. UX Conventions

## Tables

Use tables for:

- Customers
- Leads
- Representatives
- Partners
- Product Categories
- Sales

Tables should support:

- Search
- Filtering
- Sorting
- Pagination
- Empty states
- Loading states
- Error states

---

## Detail Pages

Use a consistent detail-page pattern:

```text
Header
Primary entity information
Status
Primary actions
Tabs
Related records
Activity / metadata
```

---

## Forms

Forms should:

- Place required fields first
- Minimize clicks
- Support keyboard navigation
- Provide clear validation
- Prevent duplicate submissions
- Preserve user input on validation failure
- Display success feedback after submission

---

# 33. Suggested Route Structure

## Authentication

```text
/login
/forgot-password
/reset-password
```

## Administrator

```text
/admin/dashboard

/admin/customers
/admin/customers/{id}

/admin/leads
/admin/leads/{id}

/admin/sales-representatives
/admin/sales-representatives/new
/admin/sales-representatives/{id}

/admin/partners
/admin/partners/new
/admin/partners/{id}

/admin/product-categories
/admin/product-categories/new
/admin/product-categories/{id}

/admin/reports
/admin/settings
```

## Sales Representative

```text
/sales/dashboard

/sales/customers
/sales/customers/new
/sales/customers/{id}

/sales/leads
/sales/leads/{id}
```

## Partner

```text
/partner/dashboard

/partner/leads
/partner/leads/{leadAssignmentId}

/partner/sales
/partner/sales/{id}
```

---

# 34. Recommended Phase 1 API Boundaries

The architecture should expose clean service/API boundaries even if external integrations are not yet required.

Example conceptual resources:

```text
/auth

/users
/sales-representatives
/partners
/product-categories

/customers
/leads
/lead-assignments
/sales

/dashboard
/reports
```

Future external APIs should reuse service-layer/domain logic rather than duplicating business rules.

---

# 35. Future Automatic Routing

Phase 1 uses manual assignment:

```text
Lead
 ↓
Administrator
 ↓
Select Eligible Partners
 ↓
Create LeadAssignments
```

Future phase:

```text
Lead
 ↓
Product Category
 ↓
Eligible Partners
 ↓
Routing Rules
 ↓
Automatic LeadAssignments
```

Possible future routing criteria:

- Product Category
- Partner availability
- Geography
- Lead volume limit
- Rotation / round-robin
- Partner priority
- Conversion performance
- Customer profile
- API availability
- Contract rules

Do not implement these rules in Phase 1.

Only ensure the current architecture does not prevent them.

---

# 36. Future API Integration

The Phase 1 platform should be ready for later integrations such as:

```text
External Lead Source
        ↓
      API
        ↓
    Customer
        ↓
      Lead
        ↓
Routing Engine
        ↓
LeadAssignment
        ↓
 Partner API
```

Potential future capabilities:

- Create Customer via API
- Create Lead via API
- Automatically distribute Leads
- Push Lead to Partner API
- Receive Partner status callbacks
- Receive Sale confirmation
- Webhooks
- API keys
- OAuth
- Integration logs
- Retry queue
- Dead-letter handling

These are out of scope for Phase 1.

---

# 37. Auditability

Even though a full audit system is not required, all important records should include:

```text
created_at
created_by
updated_at
updated_by
```

At minimum, track:

- Who created the Customer
- Who created the Lead
- Who assigned the Partner
- When the assignment occurred
- Which Partner updated the status
- Who registered the Sale
- When the Sale was registered

This is particularly important because the platform's business purpose involves attribution and Partner performance tracking.

---

# 38. Suggested Activity Events

A lightweight event history may be added to important detail pages.

Examples:

```text
Customer created by Eko
Lead created for Personal Loan
Lead assigned to Acme Finance
Lead assigned to ABC Finance
Acme Finance changed status to Contacted
Acme Finance changed status to In Progress
ABC Finance marked lead as No Sale
Acme Finance registered Sale
```

This can initially be implemented through an audit/event table or generated from domain timestamps.

Do not build a complex CRM activity engine unless required.

---

# 39. Dashboard Metric Definitions

To avoid ambiguity, metrics should have explicit definitions.

## Total Customers

Count of distinct active Customer records in the tenant.

```text
COUNT(Customer)
```

## Total Leads

Count of Lead records.

A Customer with three Product Categories generates three Leads.

```text
COUNT(Lead)
```

## Leads per Sales Representative

Count of Leads where:

```text
Lead.created_by_sales_rep_id = representative.id
```

## Leads per Partner Company

Count of LeadAssignments where:

```text
LeadAssignment.partner_company_id = partner.id
```

Do not count distinct Leads if the purpose is to measure distributed workload; count Partner assignments.

## Sales Registered by Partner

Count of Sale records associated with the Partner's LeadAssignments.

Optional later metric:

```text
SUM(Sale.sale_value)
```

---

# 40. Duplicate Customer Handling

Phase 1 should at minimum warn about likely duplicate Customers.

Potential matching signals:

```text
Exact phone match
Exact email match
```

Before creating a Customer, the system may show:

```text
A customer with this phone number already exists.

John Smith
+44 ...

[Open Existing Customer]
[Create Anyway]
```

Do not implement complex probabilistic entity matching in Phase 1.

---

# 41. Transactional Behavior

The `Save Customer & Submit Lead` action should ideally be atomic.

Example:

```text
BEGIN TRANSACTION

Create Customer

For each selected Product Category:
    Create Lead

COMMIT
```

If Lead creation fails, the implementation should avoid leaving the system in an inconsistent partially-created state unless explicitly handled.

Similarly, Partner assignment should avoid duplicate assignments and should provide clear error handling.

---

# 42. Recommended MVP Page Inventory

## Shared

1. Login
2. Forgot Password
3. Reset Password

## Administrator

4. Dashboard
5. Customers List
6. Customer Detail
7. Leads List
8. Lead Detail / Assignment
9. Sales Representatives List
10. Sales Representative Create/Edit
11. Sales Representative Detail
12. Partners List
13. Partner Create/Edit
14. Partner Detail
15. Product Categories List
16. Product Category Create/Edit
17. Product Category Detail
18. Reports
19. Settings

## Sales Representative

20. Dashboard
21. Customers List
22. New Customer / Submit Leads
23. Customer Detail
24. My Leads
25. Lead Detail

## Partner

26. Dashboard
27. Assigned Leads List
28. Assigned Lead Detail
29. Register Sale
30. Sales List
31. Sale Detail

Several create/edit experiences may be dialogs rather than standalone pages depending on the frontend architecture.

---

# 43. Phase 1 Acceptance Summary

Phase 1 is complete when:

### Administrator

- Can securely log in.
- Can create and manage Sales Representatives.
- Can create and manage Partner Companies.
- Can create and manage Product Categories.
- Can assign Product Categories to Partner Companies.
- Can view all Customers.
- Can identify the Sales Representative who created each Customer.
- Can view all Leads.
- Can identify the Sales Representative associated with each Lead.
- Can manually assign a Lead to one or more Partner Companies.
- Can see which Partner Companies received each Lead.
- Can view Partner status updates.
- Can view registered Sales.
- Can view the required dashboard metrics.

### Sales Representative

- Can securely log in.
- Can register a Customer.
- Can enter required Customer information.
- Can select one or more Product Categories.
- Can submit the Customer and Leads in a streamlined workflow.
- Can view Customers and Leads according to permission rules.

### Partner Company

- Can securely log in.
- Can view only LeadAssignments belonging to its own Partner Company.
- Can view complete Customer information for those Leads.
- Can update each LeadAssignment status.
- Can manually record Sale / No Sale.
- Can view own registered Sales.

### Platform

- Supports multiple Product Categories per Customer.
- Supports multiple Partner Companies per Lead.
- Keeps Customer, Lead, LeadAssignment, and Sale as separate domain concepts.
- Enforces tenant isolation.
- Enforces role permissions server-side.
- Is architected so future automatic routing and API integrations can be added without replacing the Phase 1 data model.

---

# 44. Core Architecture Decisions — Do Not Violate

These decisions should be treated as foundational constraints during implementation.

## Decision 1

```text
Customer ≠ Lead
```

A Customer may create multiple Leads.

## Decision 2

```text
Lead ≠ LeadAssignment
```

A Lead may be assigned to multiple Partner Companies.

## Decision 3

Partner processing status belongs to:

```text
LeadAssignment
```

not globally to the Lead.

## Decision 4

A Sale belongs to:

```text
LeadAssignment
```

because the Sale result is attributable to the Partner that processed the Lead.

## Decision 5

All domain data is:

```text
Tenant Scoped
```

## Decision 6

Authorization is enforced:

```text
Server Side
```

not only in frontend navigation.

## Decision 7

Phase 1 uses:

```text
Manual Lead Distribution
```

but the architecture must support future automatic routing and external APIs.

---

# 45. Final Phase 1 Domain Flow

```text
                 SALES REPRESENTATIVE
                         │
                         ▼
                      Customer
                         │
               selects one or more
                Product Categories
                         │
                         ▼
                       Leads
                         │
                         ▼
                   Administrator
                         │
                  manual assignment
                         │
                         ▼
               Lead Assignments
                 /           \
                /             \
               ▼               ▼
        Partner Company A   Partner Company B
               │               │
               ▼               ▼
            Status           Status
               │               │
          ┌────┴────┐     ┌────┴────┐
          ▼         ▼     ▼         ▼
        Sale     No Sale  Sale     No Sale
```

This model is the recommended foundation for Phase 1 implementation.
