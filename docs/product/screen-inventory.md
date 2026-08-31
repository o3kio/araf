# Prototype and MVP screen inventory

This inventory prevents hidden scope growth. It lists product surfaces, not every modal/drawer.

## Prototype screens (M0-M6)

### Tenant
- Tenant shell/home fixture
- Generic resource collection
- Generic resource details
- Schema-driven create/action flow
- Global Operations list
- Operation details

### Operator
- Operator shell/overview fixture
- Region/provider health fixture
- Project lookup/details fixture
- Global Operations list/details

### Developer/internal
- UI primitive showcase if adopted
- Descriptor/schema fixture examples

## MVP tenant screens

- Home
- Compute servers list/details/create/action subset supported upstream
- Networks list/details/action subset supported upstream
- Volumes list/details/action subset supported upstream
- Images list/details or selection flow supported upstream
- Global Resources/search entry point where backend support permits
- Operations list/details
- Projects / project context
- Users & Access supported subset
- Quotas
- Audit supported subset
- Usage & Cost supported subset
- Service Catalog
- Developer/API credentials supported subset

## MVP operator screens

- Platform overview
- Regions/AZs supported subset
- Provider/service health and capacity
- Accounts/Organizations supported subset
- Projects
- Installed Services / Service Catalog
- Global Operations
- IAM/governance supported subset
- Quota/metering/audit supported subset

## Explicitly post-MVP screens

- Billing/invoice/payment center
- Marketplace procurement
- Full branding editor
- Reseller hierarchy management
- OpenStack migration center
- VM graphical/serial console
- Network topology editor
- Kubernetes/database/DNS/load-balancer custom products
- AI assistant/control center
- Terraform generation studio
- Full provider credential editor

If implementation adds a new screen not listed for its gate, the PR must explain why it is required to satisfy an existing acceptance criterion.
