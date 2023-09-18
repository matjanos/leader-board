provider "azurerm" {
  features {}
}

variable "app_name" {
  default = "leaderboard"
}

variable "tenant_id" {
  default = "cceff820-5a2c-43eb-869a-15e1215d0b17"
}
variable "admin_object_id" {
  default = "371f4c7f-ed73-41dd-8433-6f85f04ea8b0"
}

variable "environment" {
  default = "dev"
}

variable "location" {
  default = "polandcentral"
}
variable "backup_location" {
  default = "westeurope"
}
resource "azurerm_resource_group" "resource_group" {
  name     = "rg-${var.app_name}-${var.environment}-${var.location}-001"
  location = var.location
}

resource "azurerm_key_vault" "keyvault" {
  name                = "kv-${var.app_name}-${var.environment}-001"
  resource_group_name = azurerm_resource_group.resource_group.name
  tenant_id           = var.tenant_id
  sku_name            = "standard"
  location            = var.location

  access_policy {
    tenant_id = var.tenant_id
    object_id = var.admin_object_id # object_id użytkownika

    key_permissions = [
      "Get",
      "List",
      "Update",
      "Create",
      "Import",
      "Delete",
      "Recover",
      "Backup",
      "Restore"
    ]

    secret_permissions = [
      "Get",
      "List",
      "Set",
      "Delete",
      "Recover",
      "Backup",
      "Restore"
    ]
  }
}

data "azurerm_key_vault_secret" "crossfit_user_password" {
  name         = "crossfit-user-password"
  key_vault_id = azurerm_key_vault.keyvault.id
}


resource "azurerm_container_registry" "container_registry" {
  name                = "acr${var.app_name}${var.environment}${var.location}001"
  resource_group_name = azurerm_resource_group.resource_group.name
  location            = azurerm_resource_group.resource_group.location
  sku                 = "Basic"
  admin_enabled       = false
}

resource "azurerm_container_group" "container_group" {
  name                = "acg-${var.app_name}-${var.environment}-${var.location}-001"
  location            = azurerm_resource_group.resource_group.location
  resource_group_name = azurerm_resource_group.resource_group.name
  os_type             = "Linux"
  ip_address_type     = "None"
  restart_policy      = "Never"

  identity {
    type = "SystemAssigned"
  }

  container {
    name   = "container-${var.app_name}-${var.environment}-${var.location}-001"
    image  = "mcr.microsoft.com/k8se/quickstart-jobs:latest"
    cpu    = "0.5"
    memory = "0.5"

    environment_variables = {
      "CROSSFIT_HOST"     = "https://torun.wod.guru"
      "CROSSFIT_USERNAME" = "k.matjanowski@gmail.com"
    }

    secure_environment_variables = {
      "CROSSFIT_PASSWORD" = data.azurerm_key_vault_secret.crossfit_user_password.value
    }
  }

  tags = {
    environment = var.environment
  }
}


resource "azurerm_logic_app_workflow" "logic_app_workflow" {
  name                = "law-${var.app_name}-${var.environment}-${var.location}-001"
  location            = azurerm_resource_group.resource_group.location
  resource_group_name = azurerm_resource_group.resource_group.name
}

resource "azurerm_logic_app_trigger_recurrence" "logic_app_trigger" {
  name         = "everyday"
  logic_app_id = azurerm_logic_app_workflow.logic_app_workflow.id
  frequency    = "Day"
  interval     = 1
}

resource "azurerm_key_vault_access_policy" "app_kv_access_policy" {
  key_vault_id = azurerm_key_vault.keyvault.id
  tenant_id    = azurerm_container_group.container_group.identity.0.tenant_id
  object_id    = azurerm_container_group.container_group.identity.0.principal_id
  secret_permissions = [
    "Get"
  ]
}

resource "azurerm_role_assignment" "acr_pull_access" {
  principal_id         = azurerm_container_group.container_group.identity[0].principal_id
  role_definition_name = "AcrPull"
  scope                = azurerm_container_registry.container_registry.id
}

resource "azurerm_role_assignment" "acr_admin_access" {
  principal_id         = var.admin_object_id
  role_definition_name = "Owner"
  scope                = azurerm_container_registry.container_registry.id
}